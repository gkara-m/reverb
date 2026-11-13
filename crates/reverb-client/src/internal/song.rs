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
    // params format-> ExternalType external-info
    pub fn new(params: &str) -> Result<Song, Failure> {
        match params.split_once(' ') {
            Some((external_type_as_str, external_info)) => {
                let external_type = ExternalType::get_from_str(external_type_as_str)?;
                let external_song = external_type.new_external_song(external_info)?;
                Ok(Song {
                    info: external_song.info()?,
                    song_external: external_song,
                })
            }
            None => Err(Failure::from((
                anyhow!("invalid song parameters: {}", params),
                FailureType::Warning,
            ))),
        }
    }

    pub fn default() -> Result<Song, Failure> {
        Ok(Song {
            song_external: ExternalSong::LOCAL(crate::external::local::LocalSong::new(
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

