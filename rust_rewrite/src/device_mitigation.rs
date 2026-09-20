use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::fs;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Condition {
    #[serde(rename = "operator")]
    pub op: String,
    pub value: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DeviceRule {
    pub name: Option<String>,
    pub description: Option<String>,
    pub filter_type: Option<String>,
    #[serde(default)]
    pub items: HashSet<String>,
    #[serde(default)]
    pub filter_condition: HashMap<String, Condition>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct DeviceMitigationData {
    #[serde(skip)]
    pub default_items: HashSet<String>,
    #[serde(skip)]
    pub device_rules: HashMap<String, DeviceRule>,
    #[serde(skip)]
    pub cached_mitigation_items: HashSet<String>,
}

pub struct DeviceMitigationStore {
    pub data: DeviceMitigationData,
}

impl DeviceMitigationStore {
    pub fn new() -> Self {
        Self {
            data: DeviceMitigationData::default(),
        }
    }

    pub fn load_config(&mut self, config_path: &str) -> bool {
        let content = match fs::read_to_string(config_path) {
            Ok(c) => c,
            Err(_) => return false,
        };

        let parsed: serde_json::Value = match serde_json::from_str(&content) {
            Ok(p) => p,
            Err(_) => return false,
        };

        let mut new_data = DeviceMitigationData::default();

        if let Some(default_obj) = parsed.get("default") {
            if let Some(items) = default_obj.get("items").and_then(|i| i.as_array()) {
                for item in items {
                    if let Some(s) = item.as_str() {
                        new_data.default_items.insert(s.to_string());
                    }
                }
            }
        }

        if let Some(rules_obj) = parsed.get("device_rules").and_then(|r| r.as_object()) {
            for (k, v) in rules_obj {
                if let Ok(rule) = serde_json::from_value::<DeviceRule>(v.clone()) {
                    new_data.device_rules.insert(k.clone(), rule);
                }
            }
        }

        self.data = new_data;
        true
    }

    pub fn get_cached_mitigation_items(&self, use_device_mitigation: bool) -> HashSet<String> {
        let mut items = self.data.cached_mitigation_items.clone();
        if use_device_mitigation {
            items.extend(self.data.default_items.clone());
        }
        items
    }
}
