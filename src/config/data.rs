use anyhow::anyhow;
use serde::{Deserialize, Serialize};

use crate::{
    DATA_FOLDER,
    external::{
        external::{ExternalSong, ExternalSongTrait},
        local::LocalSong,
    },
    failure::failure::{Failure, FailureType},
    internal::{
        queue::Queue,
        song::{Song, SongInfo},
    },
};

#[derive(Serialize, Deserialize)]
pub(super) struct StartupData {
    pub(super) last_played_playlist: String,
    pub(super) queue: Queue,
    pub(super) last_shutdown_clean: bool,
}

impl StartupData {
    pub(super) fn new_default() -> Result<StartupData, Failure> {
        let song = Song {
            song_type: ExternalSong::LOCAL(LocalSong::new("sample/default_song.mp3")?),
            info: SongInfo {
                title: "Default Song".to_string(),
                artists: vec!["Unknown Artist".to_string()],
            },
        };
        let startup_data = StartupData {
            last_played_playlist: "Default Playlist".to_string(),
            queue: Queue::new(song),
            last_shutdown_clean: true,
        };
        startup_data.save()?;
        Ok(startup_data)
    }

    pub(super) fn save(&self) -> Result<(), Failure> {
        match std::fs::create_dir_all(DATA_FOLDER.get().ok_or(Failure::from((
            anyhow!("DATA_FOLDER not set"),
            FailureType::Fetal,
        )))?) {
            Err(e) => return Err(Failure::from((e.into(), FailureType::Fetal))),
            Ok(_) => {}
        }
        match std::fs::write(
            format!(
                "{}startup.toml",
                DATA_FOLDER.get().ok_or(Failure::from((
                    anyhow!("DATA_FOLDER not set"),
                    FailureType::Fetal
                )))?
            ),
            toml::to_string(self).map_err(|e| Failure::from((e.into(), FailureType::Fetal)))?,
        ) {
            Err(e) => Err(Failure::from((e.into(), FailureType::Fetal))),
            Ok(_) => Ok(()),
        }
    }
}
