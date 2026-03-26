use std::{path::Path, sync::mpsc::Sender};
use anyhow::anyhow;

use crate::{CONFIG_FOLDER, CONFIG, Command, DATA_FOLDER, LOCAL_SONG_FOLDER_PATH, config::{config::Config, data::StartupData}, failure::failure::{Failure, FailureType}, internal::{internal::Internal, playlist::Playlist}};

pub fn startup() -> Result<Internal, Failure> {
    println!("Starting up... ");

    // Check for config file, create default if not exists
    println!("Reading config... ");
    let content = match std::fs::read_to_string(format!("{}config.toml", CONFIG_FOLDER)) {
        Ok(c) => Ok(c),
        Err(_) => {
            println!("Config file not found, creating default... ");
            let default = Config::new_default()?;
            toml::to_string(&default).map_err(|e| Failure::from((e.into(), FailureType::Fetal)))?;
            Err(Failure::from((anyhow!("First run?: \n Default config created in {} \n check config and restart \n exiting automatically", CONFIG_FOLDER), FailureType::Warning)))
        }
    }?;

    println!("Setting global variables... ");
    //read config
    let config: Config = toml::from_str(&content).map_err(|e| Failure::from((e.into(), FailureType::Fetal)))?;
    CONFIG.set(config).map_err(|_| Failure::from((anyhow!("Failed to set global config"), FailureType::Fetal)))?;

    // Set DATA_FOLDER
    DATA_FOLDER.set((CONFIG.get().unwrap().data_folder).clone()).map_err(|e| Failure::from((anyhow!(e), FailureType::Fetal)))?;

    let data_folder = Path::new(DATA_FOLDER.get().unwrap());
    
    // if exists, use it if not create and use
    println!("Loading startup data... ");
    let mut startup_data ;
    if data_folder.join("startup.toml").exists() {
        startup_data = toml::from_str(
            &std::fs::read_to_string(data_folder.join("startup.toml"))
            .map_err(|e| Failure::from((e.into(), FailureType::Fetal)))?
        ).map_err(|e| Failure::from((e.into(), FailureType::Fetal)))?;
    } else {
        std::fs::create_dir_all(data_folder).map_err(|e| Failure::from((e.into(), "create_dir_all failed", FailureType::Fetal)))?;
        startup_data = StartupData::new_default().map_err(|_| Failure::from((anyhow!("StartupData::new_default() Failed"), FailureType::Fetal)))?;
        println!("First run?: \n Default startup data created in {} \n no need to restart, continuing automatically\n enjoy REVERB!", data_folder.display());
    }

    if !startup_data.last_shutdown_clean {
        println!("Warning: Last shutdown was not clean, data may be corrupted, lost or incorrect. \n Attempting to continue... ");
    }

    startup_data.last_shutdown_clean = false;
    startup_data.save()?;
        

    Ok(Internal::new(startup_data.queue)?)
}

pub fn shutdown (internal: &Internal) -> Result<(), Failure> {
    println!("Shutting down... ");

    println!("Saving startup data... ");
    StartupData {
        queue: internal.queue_get().clone(),
        last_shutdown_clean: true,
    }.save()?;

    println!("Shutting down internal... ");
    internal.shutdown()?;

    Ok(())
}
