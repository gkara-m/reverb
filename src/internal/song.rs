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
    pub fn new(params: &str) -> Option<Song> {
        if let Some((external_type, external_info)) = params.split_once(' ') {
            if let Some(t) = ExternalType::get_from_str(external_type) {
                if let Some(external_song) = t.new_external_song(external_info) {
                    return Some(Song {
                        song_type: external_song,
                        title: String::from("Unknown Title"),
                        artist: String::from("Unknown Artist"),
                    });
                }
            }
        }
        None
    }
}