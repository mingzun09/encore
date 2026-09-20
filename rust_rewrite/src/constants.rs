pub const NOTIFY_TITLE: &str = "Encore Tweaks";
pub const LOG_TAG: &str = "EncoreTweaks";

pub const CONFIG_DIR: &str = "/data/adb/.config/encore";
pub const MODPATH: &str = "/data/adb/modules/encore";

pub const LOCK_FILE: &str = "/data/adb/.config/encore/.lock";
pub const JAVA_LOCK_FILE: &str = "/data/adb/.config/encore/java.lock";
pub const LOG_FILE: &str = "/data/adb/.config/encore/encore.log";
pub const PROFILE_MODE: &str = "/data/adb/.config/encore/current_profile";
pub const GAME_INFO: &str = "/data/adb/.config/encore/gameinfo";
pub const CONFIG_FILE: &str = "/data/adb/.config/encore/config.json";
pub const DEVICE_MITIGATION_FILE: &str = "/data/adb/.config/encore/device_mitigation.json";
pub const DEFAULT_CPU_GOV: &str = "/data/adb/.config/encore/default_cpu_gov";
pub const ENCORE_GAMELIST: &str = "/data/adb/.config/encore/gamelist.json";
pub const SYSTEM_STATUS_FILE: &str = "/data/adb/.config/encore/system_status";

pub const MODULE_PROP: &str = "/data/adb/modules/encore/module.prop";
pub const MODULE_UPDATE: &str = "/data/adb/modules/encore/update";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EncoreProfileMode {
    Perfcommon,
    PerformanceProfile,
    BalanceProfile,
    PowersaveProfile,
}
