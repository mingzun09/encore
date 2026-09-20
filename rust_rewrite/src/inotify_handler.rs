use inotify::{EventMask, Inotify, WatchMask};
use std::thread;

pub fn start_watcher() {
    thread::spawn(|| {
        let mut inotify = match Inotify::init() {
            Ok(i) => i,
            Err(e) => {
                log::error!("Failed to initialize inotify: {}", e);
                return;
            }
        };

        let _ = inotify
            .watches()
            .add(crate::constants::ENCORE_GAMELIST, WatchMask::CLOSE_WRITE);
        let _ = inotify
            .watches()
            .add(crate::constants::CONFIG_FILE, WatchMask::CLOSE_WRITE);
        let _ = inotify.watches().add(
            crate::constants::DEVICE_MITIGATION_FILE,
            WatchMask::CLOSE_WRITE,
        );
        let _ = inotify
            .watches()
            .add(crate::constants::MODPATH, WatchMask::CREATE);

        let mut buffer = [0; 1024];
        loop {
            match inotify.read_events_blocking(&mut buffer) {
                Ok(events) => {
                    for event in events {
                        if event.mask.contains(EventMask::CLOSE_WRITE) {
                            log::debug!("File modified, triggering reload...");
                            let mut config_store = crate::CONFIG_STORE.lock().unwrap();
                            let mut game_reg = crate::GAME_REGISTRY.lock().unwrap();
                            let mut dev_store = crate::DEVICE_MITIGATION_STORE.lock().unwrap();

                            if let Some(name) = event.name {
                                let name_str = name.to_string_lossy();
                                if name_str.ends_with("gamelist.json") {
                                    game_reg.load_from_json(crate::constants::ENCORE_GAMELIST);
                                } else if name_str.ends_with("config.json") {
                                    config_store.load_config(crate::constants::CONFIG_FILE);
                                } else if name_str.ends_with("device_mitigation.json") {
                                    dev_store.load_config(crate::constants::DEVICE_MITIGATION_FILE);
                                }
                            }
                        } else if event.mask.contains(EventMask::CREATE) {
                            if let Some(name) = event.name {
                                if name.to_string_lossy() == "update" {
                                    log::info!(
                                        "Module update file detected, signaling daemon to stop"
                                    );
                                    std::process::exit(0);
                                }
                            }
                        }
                    }
                }
                Err(e) => {
                    log::error!("Error reading inotify events: {}", e);
                    break;
                }
            }
        }
    });
}
