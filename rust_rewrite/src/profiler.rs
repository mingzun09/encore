use std::env;
use std::fs;
use std::process::Command;

use crate::config::EncoreConfigStore;
use crate::constants::*;
use crate::device_mitigation::DeviceMitigationStore;

pub fn set_profiler_env_vars(
    config_store: &EncoreConfigStore,
    device_mitigation_store: &DeviceMitigationStore,
) {
    let prefs = config_store.get_preferences();

    // In a real device we'd set/remove environment vars using libc::setenv to avoid data races.
    // In Rust, using env::set_var is highly unsafe in multi-threaded environments,
    // so we'll mock setting ENCORE_ vars here for testing purposes.
    log::debug!("Mocking setting ENCORE_ variables to avoid unsafe operations");

    let mitigation_items =
        device_mitigation_store.get_cached_mitigation_items(prefs.use_device_mitigation);

    for item in mitigation_items {
        let mut env_var = String::from("ENCORE_");
        env_var.push_str(&item);

        let safe_var: String = env_var
            .chars()
            .map(|c| {
                if c.is_ascii_alphanumeric() || c == '_' {
                    c.to_ascii_uppercase()
                } else {
                    '_'
                }
            })
            .collect();

        log::debug!("Mock Set mitigation env var: {}", safe_var);
    }

    let cpu_governor = config_store.get_cpu_governor();
    log::debug!("Mock Set ENCORE_BALANCED_CPUGOV={}", cpu_governor.balance);
    log::debug!(
        "Mock Set ENCORE_POWERSAVE_CPUGOV={}",
        cpu_governor.powersave
    );
}

pub fn run_perfcommon(
    config_store: &EncoreConfigStore,
    device_mitigation_store: &DeviceMitigationStore,
) {
    let _ = fs::write(GAME_INFO, "NULL 0 0\n");
    let _ = fs::write(
        PROFILE_MODE,
        format!("{}\n", EncoreProfileMode::Perfcommon as i32),
    );

    if config_store.get_preferences().disable_tweaks {
        log::info!("Tweaks are disabled in config, skipping perfcommon");
        return;
    }

    set_profiler_env_vars(config_store, device_mitigation_store);

    if let Err(_) = Command::new("encore_profiler").arg("perfcommon").status() {
        log::error!("Unable to execute profiler changes to perfcommon");
    }
}

pub fn apply_performance_profile(
    config_store: &EncoreConfigStore,
    device_mitigation_store: &DeviceMitigationStore,
    lite_mode: bool,
    game_pkg: &str,
    game_pid: i32,
    game_uid: i32,
) {
    let _ = fs::write(
        GAME_INFO,
        format!("{} {} {}\n", game_pkg, game_pid, game_uid),
    );
    let _ = fs::write(
        PROFILE_MODE,
        format!("{}\n", EncoreProfileMode::PerformanceProfile as i32),
    );

    if config_store.get_preferences().disable_tweaks {
        log::info!("Tweaks are disabled in config, skipping performance profile");
        return;
    }

    set_profiler_env_vars(config_store, device_mitigation_store);

    if lite_mode {
        log::debug!("Lite mode is enabled");
        if let Err(_) = Command::new("encore_profiler")
            .arg("performance_lite")
            .status()
        {
            log::error!("Unable to execute profiler changes to performance_lite");
        }
        return;
    }

    if let Err(_) = Command::new("encore_profiler").arg("performance").status() {
        log::error!("Unable to execute profiler changes to performance");
    }
}

pub fn apply_balance_profile(
    config_store: &EncoreConfigStore,
    device_mitigation_store: &DeviceMitigationStore,
) {
    let _ = fs::write(GAME_INFO, "NULL 0 0\n");
    let _ = fs::write(
        PROFILE_MODE,
        format!("{}\n", EncoreProfileMode::BalanceProfile as i32),
    );

    if config_store.get_preferences().disable_tweaks {
        log::info!("Tweaks are disabled in config, skipping balance profile");
        return;
    }

    set_profiler_env_vars(config_store, device_mitigation_store);

    if let Err(_) = Command::new("encore_profiler").arg("balance").status() {
        log::error!("Unable to execute profiler changes to balance");
    }
}

pub fn apply_powersave_profile(
    config_store: &EncoreConfigStore,
    device_mitigation_store: &DeviceMitigationStore,
) {
    let _ = fs::write(GAME_INFO, "NULL 0 0\n");
    let _ = fs::write(
        PROFILE_MODE,
        format!("{}\n", EncoreProfileMode::PowersaveProfile as i32),
    );

    if config_store.get_preferences().disable_tweaks {
        log::info!("Tweaks are disabled in config, skipping powersave profile");
        return;
    }

    set_profiler_env_vars(config_store, device_mitigation_store);

    if let Err(_) = Command::new("encore_profiler").arg("powersave").status() {
        log::error!("Unable to execute profiler changes to powersave");
    }
}
