use crate::internal::song::Song;
use crate::external::local::LocalSong;

pub trait External {
    fn play_song(&mut self, song: &Song) -> bool;

    fn pause(&self) -> bool;

    fn play(&self) -> bool;

    fn stop(&mut self) -> bool;

    fn new() -> Self;
}

pub enum ExternalType {
    LOCAL(LocalSong),
    OTHER,
}