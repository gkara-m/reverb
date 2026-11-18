use arc_swap::ArcSwap;
use serde::{Deserialize, Serialize};
use rand::random_range;
use anyhow::anyhow;
use log::{debug, info};

use reverb_core::failure::failure::{Failure, FailureType};
use once_cell::sync::OnceCell;

pub static CONFIG_FOLDER: &str = "configs/";
static CONFIG: OnceCell<ArcSwap<Config>> = OnceCell::new();


/// Config struct represents the config file.
/// do not edit data directly, use the update_config function to update the config values, this will ensure that the config is saved after updating
#[derive(Serialize, Deserialize)]
pub struct Config {
    pub data_folder: String,
    pub local_song_folder_path: String,
    pub username: String,
}

impl Config {
    fn new_default() -> Result<Config, Failure> {
        let username: String = "reverb_user_".to_string() + &random_range(0..=65535).to_string();
        let config = Config {
            data_folder: "data/".to_string(),
            local_song_folder_path: "sample/".to_string(),
            username
        };
        config.save()?;
        Ok(config)
    }

    fn save(&self) -> Result<(), Failure> {
        match std::fs::create_dir_all(CONFIG_FOLDER) {
            Err(e) => return Err(Failure::from((e.into(), FailureType::Fatal))),
            Ok(_) => {},
        }
        match std::fs::write(
            format!("{}config.toml", CONFIG_FOLDER),
            toml::to_string(self).map_err(|e| Failure::from((e.into(), FailureType::Fatal)))?,
        ) {
            Err(e) => Err(Failure::from((e.into(), FailureType::Fatal))),
            Ok(_) => Ok(()),
        }
    }

    // Check for config file, create default if not exists
    pub(super) fn load() -> Result<(), Failure> {
        info!("Reading config... ");
        let content = match std::fs::read_to_string(format!("{}config.toml", CONFIG_FOLDER)) {
            Ok(c) => Ok(c),
            Err(_) => {
                debug!("Config file not found, creating default... ");
                let default = Config::new_default()?;
                toml::to_string(&default).map_err(|e| Failure::from((e.into(), FailureType::Fatal)))?;
                Err(Failure::from((anyhow!("First run?: \n Default config created in {} \n check config and restart \n exiting automatically", CONFIG_FOLDER), FailureType::Warning)))
            }
        }?;

        info!("Setting global variables... ");
        //read config
        let config: Config = toml::from_str(&content).map_err(|e| Failure::from((e.into(), FailureType::Fatal)))?;
        CONFIG.set(ArcSwap::new(std::sync::Arc::new(config))).map_err(|_| Failure::from((anyhow!("Failed to set global config"), FailureType::Fatal)))?;
    Ok(())
    }
}

/// returns a guard to the global config, which can be used to read the config values
/// use update_config to update the config values
pub fn config() -> Result<arc_swap::Guard<std::sync::Arc<Config>>, Failure> {
    Ok(CONFIG.get().ok_or(Failure::from((anyhow!("Config not created"), FailureType::Fatal)))?.load())
}

/// updates the config values, only the fields that are Some will be updated, the rest will keep their old value
pub fn update_config(data_folder: Option<&str>, local_song_folder_path: Option<&str>, username: Option<&str>) -> Result<(), Failure> {
    // if the server config is loaded, update the fields that are Some and keep the old value for the fields that are None
    let cfg = CONFIG.get().ok_or(Failure::from((anyhow!("Config not created"), FailureType::Fatal)))?;
    cfg.rcu(|cfg| {
        let new_config = Config {
            data_folder: match data_folder {
                Some(df) => df.to_string(),
                None => cfg.data_folder.clone(),
            },
            local_song_folder_path: match local_song_folder_path {
                Some(lsf) => lsf.to_string(),
                None => cfg.local_song_folder_path.clone(),
            },
            username: match username {
                Some(u) => u.to_string(),
                None => cfg.username.clone(),
            },
        };
        std::sync::Arc::new(new_config)
    });
    config()?.save()?;
    Ok(())
}