use log::{debug, info};
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
        debug!("Initializing default startup song");
        let song = Song {
            song_external: ExternalSong::LOCAL(LocalSong::from_uri("default_song.mp3")?),
            info: SongInfo {
                title: "Default Song".to_string(),
                artists: vec!["Unknown Artist".to_string()],
            },
        };
        info!("Initialized startup song");
        let startup_data = StartupData {
            queue: Queue::new(song),
            last_shutdown_clean: true,
        };
        debug!("Initialized startup data");
        startup_data.save()?;
        info!("Saved startup data");
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
