use std::path::Path;
use std::sync::mpsc;
use std::thread;

use serde::{Serialize, Deserialize};
use toml;
use once_cell::sync::OnceCell;
use ui::cli;
use internal::{internal::Internal, song::Song};
use external::{external::ExternalSong, local::LocalSong};

use crate::internal::{playlist::Playlist, queue::Queue};

mod external;
mod ui;
mod internal;

pub static CONFIG_FOLDER: &str = "configs/";

pub static DATA_FOLDER: OnceCell<String> = OnceCell::new();
pub static LOCAL_SONG_FOLDER_PATH: OnceCell<String> = OnceCell::new();

fn main () {


    let mut internal = match startup() {
        Ok(i) => {
            println!("Startup successful!");
            i
        }
        Err(e) => {
            eprintln!("Startup error: {} \n exiting", e);
            return;
        }
    };

    let (transmit, receive) = mpsc::channel::<Command>();    

    let transmit_input = transmit.clone();
    let ui_thread = thread::spawn(
        move || {
            cli::run_cli(transmit_input);
        }
    );
    
    for command in receive {}

    loop {
        match shutdown(&internal) {
            Ok(_) => {println!("Shutdown successfull \n exiting");
            break;
        },
            Err(e) => eprintln!("Shutdown error: {} \n trying again press ^C to force exit", e),
        }
    }
}


fn startup() -> Result<Internal, String> {
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
        

    Ok(Internal::new(startup_data.queue, playlist)?)
}

fn shutdown (internal: &Internal) -> Result<(), String> {
    println!("Shutting down... ");

    println!("Saving startup data... ");
    StartupData {
        last_played_playlist: internal.playlist_get_name()?.clone(),
        queue: internal.queue_get()?.clone(),
        last_shutdown_clean: true,
    }.save()?;

    println!("Shutting down internal... ");
    internal.shutdown()?;

    Ok(())
}

// Config struct represents the config file
#[derive(Serialize, Deserialize)]
struct Config {
    data_folder: String,
    local_song_folder_path: Option<String>,
}

impl Config {
    fn new_default() -> Result<Config, String> {
        let config = Config {
            data_folder: "data/".to_string(),
            local_song_folder_path: None,
        };
        config.save()?;
        Ok(config)
    }

    fn save(&self) -> Result<(), String> {
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

#[derive(Serialize, Deserialize)]
struct StartupData {
    last_played_playlist: String,
    queue: Queue,
    last_shutdown_clean: bool,
}

impl StartupData {
    fn new_default() -> Result<StartupData, String> {
        let song = Song {
            song_type: ExternalSong::LOCAL(
                LocalSong::new("sample/default_song.mp3")?),
            title: String::from("Default Song"),
            artist: String::from("REVERB"),
        };
        let startup_data = StartupData {
            last_played_playlist: "Default Startup Playlist".to_string(),
            queue: Queue::new(song)?,
            last_shutdown_clean: true,
        };
        startup_data.save()?;
        Ok(startup_data)
    }

    fn save(&self) -> Result<(), String> {
        match std::fs::create_dir_all(DATA_FOLDER.get().ok_or("DATA_FOLDER not set".to_string())?) {
            Err(e) => return Err(format!("Failed to create data directory: {}", e)),
            Ok(_) => {},
        }
        match std::fs::write(
            format!("{}startup.toml", DATA_FOLDER.get().ok_or("DATA_FOLDER not set".to_string())?),
            toml::to_string(self).map_err(|e| format!("Failed to serialize startup data: {}", e))?,
        ) {
            Err(e) => Err(format!("Failed to write startup file: {}", e)),
            Ok(_) => Ok(()),
        }
    }
}