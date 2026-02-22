use serde::{Deserialize, Serialize};

use crate::external::external::{ExternalSong, ExternalSongTrait, ExternalType};


#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Song {
    pub song_type: ExternalSong,
    pub info: SongInfo,
}


impl Song {
    // params format-> ExternalType external-info
    pub fn new(params: &str) -> Result<Song, String> {
        match params.split_once(' ') {
            Some((external_type, external_info)) => {
                let t = ExternalType::get_from_str(external_type)?;
                let external_song = t.new_external_song(external_info)?;
                Ok(Song {
                    info: external_song.info()?,
                    song_type: external_song,
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