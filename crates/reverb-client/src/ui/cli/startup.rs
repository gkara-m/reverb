use std::path::Path;
use anyhow::anyhow;

use crate::{DATA_FOLDER, failure::failure::{Failure, FailureType}, internal::playlist::Playlist};


#[derive(serde::Deserialize, serde::Serialize)]
pub(super) struct Startup{
    pub(super) last_played_playlist: String,
}

impl Startup {
    pub(super) fn save(&self) -> Result<(), Failure> {
        match std::fs::create_dir_all(DATA_FOLDER.get().ok_or(Failure::from((
            anyhow!("DATA_FOLDER not set"),
            FailureType::Fetal,
        )))?) {
            Err(e) => return Err(Failure::from((e.into(), FailureType::Fetal))),
            Ok(_) => {}
        }
        match std::fs::write(
            Path::new(DATA_FOLDER.get().ok_or(Failure::from((
                anyhow!("DATA_FOLDER not set"),
                FailureType::Fetal,
            )))?).join("cli_startup.toml"),
            toml::to_string(self).map_err(|e| Failure::from((e.into(), FailureType::Fetal)))?,
        ) {
            Err(e) => Err(Failure::from((e.into(), FailureType::Fetal))),
            Ok(_) => Ok(()),
        }
    }

    pub(super) fn load() -> Result<Startup, Failure> {
        let path = Path::new(DATA_FOLDER.get().ok_or(Failure::from((anyhow!("DATA_FOLDER not set"), FailureType::Fetal,
        )))?).join("cli_startup.toml");
        let startup = 
        if path.exists() {
            let content = std::fs::read_to_string(&path).map_err(|e| Failure::from((e.into(), FailureType::Fetal)))?;
            toml::from_str(&content).map_err(|e| Failure::from((e.into(), FailureType::Fetal)))?
        } else {
            Playlist::new("default playlist", None)?;
            Startup {
                last_played_playlist: "default playlist".into(),
            }
        };
        match Playlist::load(&startup.last_played_playlist) {
            Ok(_) => {},
            Err(e) => {
                println!("Failed to load last played playlist '{}', error: {}. Attempting to continue with default playlist", startup.last_played_playlist, e);
                Playlist::new(&startup.last_played_playlist, None)?;
            }
        }
        startup.save()?;
        Ok(startup)
    }
}