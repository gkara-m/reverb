use serde::{Deserialize, Serialize};

use crate::{CONFIG_FOLDER, failure::failure::{Failure, FailureType}};

// Config struct represents the config file
#[derive(Serialize, Deserialize)]
pub struct Config {
    // pub device_name: String,
    pub(super) data_folder: String,
    pub(super) local_song_folder_path: Option<String>,
}

impl Config {
    pub(super) fn new_default() -> Result<Config, Failure> {
        let config = Config {
            // device_name: "REVERB_user".to_string(),
            data_folder: "data/".to_string(),
            local_song_folder_path: None,
        };
        config.save()?;
        Ok(config)
    }

    pub(super) fn save(&self) -> Result<(), Failure> {
        match std::fs::create_dir_all(CONFIG_FOLDER) {
            Err(e) => return Err(Failure::from((e.into(), FailureType::Fetal))),
            Ok(_) => {},
        }
        match std::fs::write(
            format!("{}config.toml", CONFIG_FOLDER),
            toml::to_string(self).map_err(|e| Failure::from((e.into(), FailureType::Fetal)))?,
        ) {
            Err(e) => Err(Failure::from((e.into(), FailureType::Fetal))),
            Ok(_) => Ok(()),
        }
    }
}