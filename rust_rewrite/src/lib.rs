pub mod binder_monitor;
pub mod binder_ndk;
pub mod config;
pub mod constants;
pub mod device_mitigation;
pub mod game_registry;
pub mod inotify_handler;
pub mod profiler;
pub use crate::config::EncoreConfigStore;
pub use crate::device_mitigation::DeviceMitigationStore;
pub use crate::game_registry::GameRegistry;
use std::sync::Mutex;
lazy_static::lazy_static! { pub static ref GAME_REGISTRY: Mutex<GameRegistry> = Mutex::new(GameRegistry::new()); pub static ref CONFIG_STORE: Mutex<EncoreConfigStore> = Mutex::new(EncoreConfigStore::new()); pub static ref DEVICE_MITIGATION_STORE: Mutex<DeviceMitigationStore> = Mutex::new(DeviceMitigationStore::new()); }
