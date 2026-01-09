use crate::internal::song::Song;
use crate::external::local::{Local, LocalSong};

pub trait External {
    fn play_new(&self, song: &Song) -> bool;

    fn pause(&self) -> bool;

    fn play(&self) -> bool;

    fn stop(&self) -> bool;

}

pub enum ExternalSongType {
    LOCAL(LocalSong),
    YOUTUBE(String),
}