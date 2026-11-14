use anyhow::anyhow;
use serde::{Deserialize, Serialize};

use crate::{
    external::external::{ExternalSong, ExternalSongTrait, ExternalType}, internal::playlist::m3u::M3USongInfo,
};
use reverb_core::failure::failure::{Failure, FailureType};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Song {
    pub song_external: ExternalSong,
    pub info: SongInfo,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SongInfo {
    pub title: String,
    pub artists: Vec<String>,
}

impl Song {
    // params format-> uri
    pub fn new(uri: &str) -> Result<Song, Failure> {
            let external_type = ExternalType::get_from_uri(uri)?;
            let external_song = external_type.new_external_song(uri)?;
            Ok(Song {
                info: external_song.info()?,
                song_external: external_song,
            })
    }

    pub fn default() -> Result<Song, Failure> {
        Ok(Song {
            song_external: ExternalSong::LOCAL(crate::external::local::LocalSong::from_uri(
                "sample/default_song.mp3",
            )?),
            info: SongInfo {
                title: "Default Song".to_string(),
                artists: vec!["REVERB".to_string()],
            },
        })
    }

    pub fn to_m3u_song_info(&self) -> Result<M3USongInfo, Failure> {
        self.song_external.to_m3u_song_info()
    }
}

