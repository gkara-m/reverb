use serde::{Deserialize, Serialize};

use crate::{
    config::config::config, external::{
        external::{ExternalSong, ExternalSongTrait},
        local::LocalSong,
    }, internal::{
        queue::Queue,
        song::{Song, SongInfo},
    }
};

use reverb_core::failure::failure::{Failure, FailureType};

#[derive(Serialize, Deserialize)]
pub(super) struct StartupData {
    pub(super) queue: Queue,
    pub(super) last_shutdown_clean: bool,
}

impl StartupData {
    pub(super) fn new_default() -> Result<StartupData, Failure> {
        println!("Attempting to initialize Startup Song");
        let song = Song {
            song_external: ExternalSong::LOCAL(LocalSong::new("default_song.mp3")?),
            info: SongInfo {
                title: "Default Song".to_string(),
                artists: vec!["Unknown Artist".to_string()],
            },
        };
        println!("Successfully initialized Startup Song");
        let startup_data = StartupData {
            queue: Queue::new(song),
            last_shutdown_clean: true,
        };
        println!("Successfully initialized StartupData");
        startup_data.save()?;
        println!("Successfully saved StartupData");
        Ok(startup_data)
    }

    pub(super) fn save(&self) -> Result<(), Failure> {
        match std::fs::create_dir_all(config()?.data_folder.clone()) {
            Err(e) => return Err(Failure::from((e.into(), FailureType::Fatal))),
            Ok(_) => {}
        }
        match std::fs::write(
            format!(
                "{}startup.toml",
                config()?.data_folder
            ),
            toml::to_string(self).map_err(|e| Failure::from((e.into(), FailureType::Fatal)))?,
        ) {
            Err(e) => Err(Failure::from((e.into(), FailureType::Fatal))),
            Ok(_) => Ok(()),
        }
    }
}
