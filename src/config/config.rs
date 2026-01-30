use serde::{Deserialize, Serialize};

use crate::CONFIG_FOLDER;

// Config struct represents the config file
#[derive(Serialize, Deserialize)]
pub(super) struct Config {
    pub(super) data_folder: String,
    pub(super) local_song_folder_path: Option<String>,
}

impl Config {
    pub fn new_default() -> Result<Config, String> {
        let config = Config {
            data_folder: "data/".to_string(),
            local_song_folder_path: None,
        };
        config.save()?;
        Ok(config)
    }

    pub fn save(&self) -> Result<(), String> {
        match std::fs::create_dir_all(CONFIG_FOLDER) {
            Err(e) => return Err(format!("Failed to create config directory: {}", e)),
            Ok(_) => {},
        }
        match std::fs::write(
            format!("{}config.toml", CONFIG_FOLDER),
            toml::to_string(self).map_err(|e| format!("Failed to serialize config: {}", e))?,
        ) {
            Err(e) => Err(format!("Failed to write config file: {}", e)),
            Ok(_) => Ok(()),
        }
    }
}