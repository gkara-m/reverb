use std::path::Path;
use std::sync::mpsc;
use std::thread;

use serde::{Serialize, Deserialize};
use toml;
use once_cell::sync::OnceCell;
use ui::cli;
use internal::{internal::Internal, song::Song};
use external::{external::ExternalSong, local::LocalSong};

use crate::{external::external::{ExternalSongTrait, ExternalType}, internal::{playlist::Playlist, queue::Queue}};

mod external;
mod ui;
mod internal;

pub static CONFIG_FOLDER: &str = "configs/";

pub static DATA_FOLDER: OnceCell<String> = OnceCell::new();
pub static LOCAL_SONG_FOLDER_PATH: OnceCell<String> = OnceCell::new();

fn main () {

    let (transmit, receive) = mpsc::channel::<Command>();

    let mut internal = match startup(transmit.clone()) {
        Ok(i) => {
            println!("Startup successful!");
            i
        }
        Err(e) => {
            eprintln!("Startup error: {} \n exiting", e);
            return;
        }
    };


    let ui_thread = thread::spawn(
        move || {
            cli::run_cli(transmit);
        }
    );
    
    for command in receive {
        if let Err(e) = match command {
            Command::Play => internal.play(),
            Command::Pause => internal.pause(),
            Command::PlayNew(song) => internal.play_new(song),
            Command::Stop => Err("Stop command not implemented".to_string()),
            Command::CurrentSong(sender) => {
                match internal.current_song() {
                    Ok(song) => sender.send(song).map_err(|e| format!("Failed to send current song: {}", e)),
                    Err(e) => Err(e),
                }
            },
            Command::PlaylistLoad(name) => internal.playlist_load(&name),
            Command::PlaylistNew { name, external_type } => internal.playlist_new(&name, external_type),
            Command::PlaylistAdd(song) => internal.playlist_add(song),
            Command::PlaylistRemove(index) => internal.playlist_remove(index),
            Command::PlaylistMoveSong { from, to } => internal.playlist_move_song(from, to),
            Command::PlaylistGetSongs(sender) => {
                match internal.playlist_get_songs() {
                    Ok(songs) => sender.send(songs).map_err(|e| format!("Failed to send playlist songs: {}", e)),
                    Err(e) => Err(e),
                }
            },
            Command::PlaylistGetName(sender) => {
                match internal.playlist_get_name() {
                    Ok(name) => sender.send(name).map_err(|e| format!("Failed to send playlist name: {}", e)),
                    Err(e) => Err(e),
                }
            },
            Command::PlaylistSetName(name) => internal.playlist_set_name(name.as_str()),
            Command::PlaylistGetSong { song, index } => {
                match internal.playlist_get_song(index) {
                    Ok(s) => song.send(s).map_err(|e| format!("Failed to send playlist song: {}", e)),
                    Err(e) => Err(e),
                }
            },
            Command::QueueAdd(song) => internal.queue_add(song),
            Command::QueueRemove(index) => internal.queue_remove(index),
            Command::QueueList => internal.queue_list(),
            Command::QueueNext => internal.queue_next(),
            Command::QueuePlaylist(playlist) => internal.queue_playlist(&playlist),
            Command::QueueCurrentPlaylist => internal.queue_current_playlist(),
            Command::QueueGet(sender) => {
                match internal.queue_get() {
                    Ok(queue) => sender.send(queue.clone()).map_err(|e| format!("Failed to send queue: {}", e)),
                    Err(e) => Err(e),
                }
            },
            Command::Shutdown => break,
        }  {
            cli::invalid_input(e);
        }
    }

    loop {
        match shutdown(&internal) {
            Ok(_) => {println!("Shutdown successfull \n exiting");
            break;
        },
            Err(e) => eprintln!("Shutdown error: {} \n trying again press ^C to force exit", e),
        }
    }
}


fn startup(transmit: std::sync::mpsc::Sender<crate::Command>) -> Result<Internal, String> {
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
            info: crate::internal::song::SongInfo {
                title: String::from("Default Song"),
                artist: String::from("REVERB"),
            }
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

enum Command {
    Play,
    Pause,
    PlayNew(Song),
    Stop,
    CurrentSong(mpsc::Sender<Song>),
    PlaylistLoad(String),
    PlaylistNew {
        name: String,
        external_type: Option<ExternalType>,
    },
    PlaylistAdd(Song),
    PlaylistRemove(usize),
    PlaylistMoveSong {
        from: usize,
        to: usize,
    },
    PlaylistGetSongs(mpsc::Sender<Vec<Song>>),
    PlaylistGetName(mpsc::Sender<String>),
    PlaylistSetName(String),
    PlaylistGetSong {
        song: mpsc::Sender<Song>,
        index: usize,
    },
    QueueAdd(Song),
    QueueRemove(usize),
    QueueList,
    QueueNext,
    QueuePlaylist(Playlist),
    QueueCurrentPlaylist,
    QueueGet(mpsc::Sender<Queue>),
    Shutdown,
}