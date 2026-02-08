use serde::Deserialize;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

#[derive(Deserialize)]
struct ConfigFile {
    mappings: HashMap<String, String>,
}

pub struct Config {
    pub mappings: HashMap<String, String>,
    pub max_trigger_len: usize,
}

impl Config {
    pub fn load() -> Self {
        let path = Self::config_path();
        let text = std::fs::read_to_string(&path)
            .unwrap_or_else(|e| panic!("failed to read config at {}: {e}", path.display()));
        let file: ConfigFile =
            toml::from_str(&text).unwrap_or_else(|e| panic!("invalid config: {e}"));

        let max_trigger_len = file.mappings.keys().map(|k| k.len()).max().unwrap_or(0);

        Self {
            mappings: file.mappings,
            max_trigger_len,
        }
    }

    fn config_path() -> PathBuf {
        let candidates = [
            std::env::current_dir()
                .ok()
                .map(|d| d.join("umlauter.toml")),
            dirs::config_dir().map(|d| d.join("umlauter/umlauter.toml")),
            dirs::home_dir().map(|d| d.join(".config/umlauter/umlauter.toml")),
        ];

        for candidate in candidates.into_iter().flatten() {
            if Path::new(&candidate).exists() {
                return candidate;
            }
        }

        panic!("no umlauter.toml found in cwd, ~/.config/umlauter/, or XDG config dir");
    }
}
