use anyhow::anyhow;
use serde::{Deserialize, Serialize};

use crate::{
    external::external::{ExternalSong, ExternalSongTrait, ExternalType},
};
use reverb_core::failure::failure::{Failure, FailureType};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Song {
    pub song_type: ExternalSong,
    pub info: SongInfo,
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
                    song_type: external_song,
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
            song_type: ExternalSong::LOCAL(crate::external::local::LocalSong::new(
                "sample/default_song.mp3",
            )?),
            info: SongInfo {
                title: "Default Song".to_string(),
                artists: vec!["REVERB".to_string()],
            },
        })
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SongInfo {
    pub title: String,
    pub artists: Vec<String>,
}
