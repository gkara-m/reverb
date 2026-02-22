use std::{path::Path, sync::mpsc::Sender};

use crate::{CONFIG_FOLDER, Command, DATA_FOLDER, LOCAL_SONG_FOLDER_PATH, config::{config::Config, data::StartupData}, internal::{internal::Internal, playlist::Playlist}};

pub fn startup(transmit: Sender<Command>) -> Result<Internal, String> {
    println!("Starting up... ");

    // Check for config file, create default if not exists
    println!("Reading config... ");
    let content = match std::fs::read_to_string(format!("{}config.toml", CONFIG_FOLDER)) {
        Ok(c) => Ok(c),
        Err(_) => {
            println!("Config file not found, creating default... ");
            let default = Config::new_default()?;
            toml::to_string(&default).map_err(|e| format!("Failed to make default config: {}", e))?;
            Err(format!("First run?: \n Default config created in {} \n check config and restart \n exiting automatically", CONFIG_FOLDER))
        }
    }?;

    println!("Setting global variables... ");
    //read config
    let config: Config = toml::from_str(&content).map_err(|e| format!("Failed to read config file: {}", e))?;

    // Set DATA_FOLDER
    DATA_FOLDER.set(config.data_folder.clone()).map_err(|_| "DATA_FOLDER already set".to_string())?;

    // Set LOCAL_SONG_FOLDER_PATH if provided in config
    if let Some(local_path) = config.local_song_folder_path {
        LOCAL_SONG_FOLDER_PATH.set(local_path).map_err(|_| "LOCAL_SONG_FOLDER_PATH already set".to_string())?;
    }

    let data_folder = Path::new(DATA_FOLDER.get().unwrap());
    
    // if exists, use it if not create and use
    println!("Loading startup data... ");
    let mut startup_data ;
    if data_folder.join("startup.toml").exists() {
        startup_data = toml::from_str(
            &std::fs::read_to_string(data_folder.join("startup.toml"))
            .map_err(|e| format!("Failed to read startup data: {}", e))?
        ).map_err(|e| format!("Failed to parse startup data: {}", e))?;
    } else {
        std::fs::create_dir_all(data_folder).map_err(|e| format!("Failed to create data folder: {}", e))?;
        startup_data = StartupData::new_default()?;
        println!("First run?: \n Default startup data created in {} \n no need to restart, continuing automatically\n enjoy REVERB!", data_folder.display());
    }

    if !startup_data.last_shutdown_clean {
        println!("Warning: Last shutdown was not clean, data may be corrupted, lost or incorrect. \n Attempting to continue... ");
    }

    startup_data.last_shutdown_clean = false;
    startup_data.save()?;

    // check if last played playlist exists, if not create default
    let playlist =
    if data_folder.join("playlists").join(format!("{}.json", startup_data.last_played_playlist)).exists() {
        Playlist::load(&startup_data.last_played_playlist)?
    } else {
        println!("Last played playlist not found, creating default playlist... ");
        Playlist::new("Default Playlist", None)?
    };
        

    Ok(Internal::new(startup_data.queue, playlist, transmit)?)
}

pub fn shutdown (internal: &Internal) -> Result<(), String> {
    println!("Shutting down... ");

    println!("Saving startup data... ");
    StartupData {
        last_played_playlist: internal.playlist_get_name().clone(),
        queue: internal.queue_get().clone(),
        last_shutdown_clean: true,
    }.save()?;

    println!("Shutting down internal... ");
    internal.shutdown()?;

    Ok(())
}