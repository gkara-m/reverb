use serde::{Deserialize, Serialize};

use crate::{DATA_FOLDER, external::{external::{ExternalSong, ExternalSongTrait}, local::LocalSong}, internal::{queue::Queue, song::{Song, SongInfo}}};

#[derive(Serialize, Deserialize)]
pub(super) struct StartupData {
    pub(super) last_played_playlist: String,
    pub(super) queue: Queue,
    pub(super) last_shutdown_clean: bool,
}

impl StartupData {
    pub(super) fn new_default() -> Result<StartupData, String> {
        let song = Song {
            song_type: ExternalSong::LOCAL(
                LocalSong::new("sample/default_song.mp3")?),
                    info: SongInfo {
                        title: "Default Song".to_string(),
                        artist: "Unknown Artist".to_string(),
                    },
        };
        let startup_data = StartupData {
            last_played_playlist: "Default Startup Playlist".to_string(),
            queue: Queue::new(song)?,
            last_shutdown_clean: true,
        };
        startup_data.save()?;
        Ok(startup_data)
    }

    pub(super) fn save(&self) -> Result<(), String> {
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