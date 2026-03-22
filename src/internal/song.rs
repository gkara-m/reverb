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
            Some((external_type_as_str, external_info)) => {
                let external_type = ExternalType::get_from_str(external_type_as_str)?;
                let external_song = external_type.new_external_song(external_info)?;
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

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SongInfo {
    pub title: String,
    pub artist: String,
}

