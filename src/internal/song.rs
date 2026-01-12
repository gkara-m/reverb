use serde::{Deserialize, Serialize};

use crate::external::external::{ExternalSong, ExternalType};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Song {
    pub song_type: ExternalSong,
    pub title: String,
    pub artist: String,
}

impl Song {
    // params format-> ExternalType external-info
    pub fn new(params: &str) -> Result<Song, String> {
        match params.split_once(' ') {
            Some((external_type, external_info)) => {
                let t = ExternalType::get_from_str(external_type)?;
                let external_song = t.new_external_song(external_info)?;
                Ok(Song {
                    song_type: external_song,
                    title: String::from("Unknown Title"),
                    artist: String::from("Unknown Artist"),
                })
            }
            None => Err(format!("invalid song parameters: {}", params)),
        }
    }
}