use crate::external::external::ExternalSongType;

pub struct Song {
    pub song_type: ExternalSongType,
    pub title: String,
    pub artist: String,
}