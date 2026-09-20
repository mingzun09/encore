use daemonize::Daemonize;
use encored::constants::*;
use lazy_static::lazy_static;
use std::process;
use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, Ordering};

use encored::binder_monitor;
use encored::config::EncoreConfigStore;
use encored::device_mitigation::DeviceMitigationStore;
use encored::game_registry::GameRegistry;
use encored::inotify_handler;
use encored::profiler;
use std::env;

pub struct DaemonState {
    pub cur_mode: EncoreProfileMode,
    pub active_package: String,
    pub active_game_pid: i32,
    pub active_game_uid: i32,
    pub last_applied_pid: i32,
    pub screen_awake: bool,
    pub battery_saver_state: bool,
    pub game_requested_dnd: bool,
    pub prev_dnd_state: bool,
}

lazy_static! {
    pub static ref G_STATE: Mutex<DaemonState> = Mutex::new(DaemonState {
        cur_mode: EncoreProfileMode::Perfcommon,
        active_package: String::new(),
        active_game_pid: 0,
        active_game_uid: 0,
        last_applied_pid: 0,
        screen_awake: true,
        battery_saver_state: false,
        game_requested_dnd: false,
        prev_dnd_state: false,
    });
    pub static ref GAME_REGISTRY: Mutex<GameRegistry> = Mutex::new(GameRegistry::new());
    pub static ref CONFIG_STORE: Mutex<EncoreConfigStore> = Mutex::new(EncoreConfigStore::new());
    pub static ref DEVICE_MITIGATION_STORE: Mutex<DeviceMitigationStore> =
        Mutex::new(DeviceMitigationStore::new());
}

#[allow(dead_code)]
static DAEMON_STOP_REQUESTED: AtomicBool = AtomicBool::new(false);

#[allow(dead_code)]
fn signal_daemon_stop() {
    DAEMON_STOP_REQUESTED.store(true, Ordering::Relaxed);
    process::exit(0);
}

fn evaluate_and_apply_profile() {
    let mut state = G_STATE.lock().unwrap();
    let game_reg = encored::GAME_REGISTRY.lock().unwrap();
    let config_store = encored::CONFIG_STORE.lock().unwrap();
    let dev_store = encored::DEVICE_MITIGATION_STORE.lock().unwrap();

    if !state.game_requested_dnd {
        state.prev_dnd_state = false;
    }

    if state.active_game_pid != 0 && state.screen_awake {
        if state.cur_mode == EncoreProfileMode::PerformanceProfile
            && state.last_applied_pid == state.active_game_pid
        {
            return;
        }

        if let Some(game) = game_reg.find_game(&state.active_package) {
            state.cur_mode = EncoreProfileMode::PerformanceProfile;
            log::info!(
                "Applying performance profile for {} (PID: {})",
                state.active_package,
                state.active_game_pid
            );

            let lite_mode = game.lite_mode || config_store.get_preferences().enforce_lite_mode;
            profiler::apply_performance_profile(
                &config_store,
                &dev_store,
                lite_mode,
                &state.active_package,
                state.active_game_pid,
                state.active_game_uid,
            );

            state.last_applied_pid = state.active_game_pid;
            return;
        }
    }

    if state.battery_saver_state {
        if state.cur_mode == EncoreProfileMode::PowersaveProfile {
            return;
        }
        state.cur_mode = EncoreProfileMode::PowersaveProfile;
        state.last_applied_pid = 0;
        state.active_game_uid = 0;
        log::info!("Applying powersave profile");
        profiler::apply_powersave_profile(&config_store, &dev_store);
        return;
    }

    if state.cur_mode == EncoreProfileMode::BalanceProfile {
        return;
    }
    state.cur_mode = EncoreProfileMode::BalanceProfile;
    state.last_applied_pid = 0;
    state.active_game_uid = 0;
    log::info!("Applying balance profile");
    profiler::apply_balance_profile(&config_store, &dev_store);
}

fn run_daemon() -> i32 {
    let mut config_store = encored::CONFIG_STORE.lock().unwrap();
    if !config_store.load_config(CONFIG_FILE) {
        log::error!("Failed to parse config file");
        return 1;
    }
    drop(config_store);

    let mut game_reg = encored::GAME_REGISTRY.lock().unwrap();
    if !game_reg.load_from_json(ENCORE_GAMELIST) {
        log::error!("Failed to parse gamelist.json");
        return 1;
    }
    drop(game_reg);

    let mut dev_store = encored::DEVICE_MITIGATION_STORE.lock().unwrap();
    if !dev_store.load_config(DEVICE_MITIGATION_FILE) {
        log::error!("Failed to parse device_mitigation.json");
        return 1;
    }
    drop(dev_store);

    let daemonize = Daemonize::new().working_directory("/");
    match daemonize.start() {
        Ok(_) => log::info!("Daemonized successfully"),
        Err(_) => {
            log::error!("Failed to daemonize");
            return 1;
        }
    }

    let config_store = encored::CONFIG_STORE.lock().unwrap();
    let dev_store = encored::DEVICE_MITIGATION_STORE.lock().unwrap();
    profiler::run_perfcommon(&config_store, &dev_store);
    drop(config_store);
    drop(dev_store);

    if !binder_monitor::initialize() {
        log::error!("Failed to initialize BinderMonitor");
        return 1;
    }

    inotify_handler::start_watcher();

    binder_monitor::set_process_observer_callbacks(binder_monitor::ProcessObserverCallbacks {
        on_foreground_activities_changed: Some(Box::new(|pid, uid, foreground| {
            if !foreground {
                return;
            }
            let pkg = binder_monitor::get_package_name_for_uid(uid);
            if pkg.is_empty() {
                return;
            }
            let game_reg = encored::GAME_REGISTRY.lock().unwrap();
            if !game_reg.is_game_registered(&pkg) {
                return;
            }
            drop(game_reg);

            let mut state = G_STATE.lock().unwrap();
            if state.active_game_pid != pid {
                state.active_package = pkg.clone();
                state.active_game_pid = pid;
                state.active_game_uid = uid;
                log::info!("Game {} came to foreground (PID: {})", pkg, pid);
                drop(state);
                evaluate_and_apply_profile();
            }
        })),
        on_process_died: Some(Box::new(|pid, _uid| {
            let mut state = G_STATE.lock().unwrap();
            if state.active_game_pid == pid {
                log::info!(
                    "Game {} (PID: {}) exited, resetting profile",
                    state.active_package,
                    pid
                );
                state.active_package.clear();
                state.active_game_pid = 0;
                state.last_applied_pid = 0;
                drop(state);
                evaluate_and_apply_profile();
            }
        })),
    });

    binder_monitor::set_display_state_callback(Box::new(|is_interactive| {
        let mut state = G_STATE.lock().unwrap();
        state.screen_awake = is_interactive;
        drop(state);
        evaluate_and_apply_profile();
    }));

    evaluate_and_apply_profile();
    log::info!("Encore Tweaks daemon started");

    binder_monitor::join_thread_pool();
    0
}

fn cmd_setup_gamelist(base_file_path: &str) -> i32 {
    let mut game_reg = encored::GAME_REGISTRY.lock().unwrap();
    if game_reg.load_from_json(base_file_path) {
        println!("Successfully parsed gamelist from {}", base_file_path);
        0
    } else {
        eprintln!("ERROR: Failed to setup gamelist from {}", base_file_path);
        1
    }
}

fn cmd_check_gamelist() -> i32 {
    let mut game_reg = encored::GAME_REGISTRY.lock().unwrap();
    if game_reg.load_from_json(ENCORE_GAMELIST) {
        eprintln!("{} is valid", ENCORE_GAMELIST);
        eprintln!("Registered games: {}", game_reg.games.len());
        0
    } else {
        eprintln!("ERROR: Failed to parse {}", ENCORE_GAMELIST);
        1
    }
}

fn print_usage(prog: &str) {
    println!("Encore Tweaks CLI\n");
    println!("Usage: {} <COMMAND> [OPTIONS]\n", prog);
    println!("Commands:");
    println!("  daemon               Start Encore Tweaks daemon");
    println!("  setup_gamelist       Setup initial gamelist from base file");
    println!("  check_gamelist       Validate gamelist file");
    println!("  version              Show version information\n");
    println!("Global Options:");
    println!("  -h, --help           Show this help message");
    println!("  -V, --version        Show version information\n");
}

fn main() {
    env_logger::init();
    let args: Vec<String> = env::args().collect();
    let prog = if args.len() > 0 { &args[0] } else { "encored" };

    if args.len() < 2 {
        print_usage(prog);
        process::exit(1);
    }

    let cmd = &args[1];

    if cmd == "-h" || cmd == "--help" {
        print_usage(prog);
        process::exit(0);
    } else if cmd == "-V" || cmd == "--version" || cmd == "version" {
        println!("Encore Tweaks Rust Rewrite");
        process::exit(0);
    } else if cmd == "daemon" {
        let code = run_daemon();
        process::exit(code);
    } else if cmd == "setup_gamelist" {
        if args.len() != 3 {
            eprintln!("ERROR: Invalid arguments.");
            println!("Usage: {} setup_gamelist <base_file_path>\n", prog);
            process::exit(1);
        }
        process::exit(cmd_setup_gamelist(&args[2]));
    } else if cmd == "check_gamelist" {
        process::exit(cmd_check_gamelist());
    } else {
        eprintln!("ERROR: Unknown command: {}", cmd);
        print_usage(prog);
        process::exit(1);
    }
}
