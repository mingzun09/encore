use serde::{Deserialize, Serialize};
use std::fs;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Preferences {
    #[serde(default)]
    pub enforce_lite_mode: bool,
    #[serde(default)]
    pub use_device_mitigation: bool,
    #[serde(default)]
    pub disable_tweaks: bool,
    #[serde(default = "default_log_level")]
    pub log_level: i32,
}

fn default_log_level() -> i32 {
    4
}

impl Default for Preferences {
    fn default() -> Self {
        Self {
            enforce_lite_mode: false,
            use_device_mitigation: false,
            disable_tweaks: false,
            log_level: 4,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct CPUGovernor {
    #[serde(default)]
    pub balance: String,
    #[serde(default)]
    pub powersave: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct ConfigData {
    #[serde(default)]
    pub preferences: Preferences,
    #[serde(default)]
    pub cpu_governor: CPUGovernor,
}

pub struct EncoreConfigStore {
    pub config: ConfigData,
    pub config_path: String,
}

impl EncoreConfigStore {
    pub fn new() -> Self {
        Self {
            config: ConfigData::default(),
            config_path: String::new(),
        }
    }

    pub fn load_config(&mut self, config_path: &str) -> bool {
        self.config_path = config_path.to_string();
        let content = match fs::read_to_string(config_path) {
            Ok(c) => c,
            Err(_) => return false,
        };

        let parsed: ConfigData = match serde_json::from_str(&content) {
            Ok(p) => p,
            Err(_) => return false,
        };

        self.config = parsed;
        true
    }

    pub fn get_preferences(&self) -> Preferences {
        self.config.preferences.clone()
    }

    pub fn get_cpu_governor(&self) -> CPUGovernor {
        self.config.cpu_governor.clone()
    }

    pub fn reload(&mut self) -> bool {
        let path = self.config_path.clone();
        if !path.is_empty() {
            self.load_config(&path)
        } else {
            false
        }
    }
}
