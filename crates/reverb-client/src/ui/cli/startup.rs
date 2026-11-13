use std::path::Path;

use reverb_core::failure::failure::{Failure, FailureType};

use crate::{config::config::config, internal::playlist::playlist::Playlist};

#[derive(serde::Deserialize, serde::Serialize)]
pub(super) struct Startup{
    pub(super) last_played_playlist: String,
}

impl Startup {
    pub(super) fn save(&self) -> Result<(), Failure> {
        match std::fs::create_dir_all(config()?.data_folder.clone()) {
            Err(e) => return Err(Failure::from((e.into(), FailureType::Fatal))),
            Ok(_) => {}
        }
        match std::fs::write(
            Path::new(&config()?.data_folder.clone()).join("cli_startup.toml"),
            toml::to_string(self).map_err(|e| Failure::from((e.into(), FailureType::Fatal)))?,
        ) {
            Err(e) => Err(Failure::from((e.into(), FailureType::Fatal))),
            Ok(_) => Ok(()),
        }
    }

    pub(super) fn load() -> Result<Startup, Failure> {
        println!("Loading startup playlist...");
        println!("Loading last played playlist...");
        let path = Path::new(&config()?.data_folder.clone()).join("cli_startup.toml");
        println!("Startup file path: {:?}", path);
        let startup = 
        if path.exists() {
            println!("Startup file exists, loading...");
            let content = std::fs::read_to_string(&path).map_err(|e| Failure::from((e.into(), FailureType::Fatal)))?;
            println!("Startup file content: {}", content);
            toml::from_str(&content).map_err(|e| Failure::from((e.into(), FailureType::Fatal)))?
        } else {
            println!("Startup file does not exist, creating default startup...");
            Playlist::new("default playlist", None)?;
            println!("Default playlist created.");
            Startup {
                last_played_playlist: "default playlist".into(),
            }
        };
        println!("Startup loaded: last played playlist: {}", startup.last_played_playlist);
        match Playlist::load(&startup.last_played_playlist) {
            Ok(_) => {println!("Last played playlist '{}' loaded successfully.", startup.last_played_playlist);},
            Err(e) => {
                println!("Failed to load last played playlist '{}', error: {}. Attempting to continue with default playlist", startup.last_played_playlist, e);
                Playlist::new(&startup.last_played_playlist, None)?;
            }
        }
        startup.save()?;
        println!("Startup saved.");
        Ok(startup)
    }
}
