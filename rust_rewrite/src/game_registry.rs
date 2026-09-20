use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct EncoreGameList {
    #[serde(skip)]
    pub package_name: String,

    #[serde(default)]
    pub lite_mode: bool,

    #[serde(default)]
    pub enable_dnd: bool,
}

pub struct GameRegistry {
    pub games: HashMap<String, EncoreGameList>,
}

impl GameRegistry {
    pub fn new() -> Self {
        Self {
            games: HashMap::new(),
        }
    }

    pub fn load_from_json(&mut self, path: &str) -> bool {
        let content = match fs::read_to_string(path) {
            Ok(c) => c,
            Err(_) => return false,
        };

        let parsed: HashMap<String, EncoreGameList> = match serde_json::from_str(&content) {
            Ok(p) => p,
            Err(_) => return false,
        };

        self.games.clear();
        for (k, mut v) in parsed {
            v.package_name = k.clone();
            self.games.insert(k, v);
        }

        true
    }

    pub fn is_game_registered(&self, package_name: &str) -> bool {
        self.games.contains_key(package_name)
    }

    pub fn find_game(&self, package_name: &str) -> Option<EncoreGameList> {
        self.games.get(package_name).cloned()
    }
}
