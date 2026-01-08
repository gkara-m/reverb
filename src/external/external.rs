use crate::internal::song;
use crate::external::local::LocalSong;

pub trait External {
    fn play_song(&mut self, song: song::Song) -> bool;

    fn pause(&self) -> bool;

    fn play(&self) -> bool;

    fn stop(&mut self) -> Option<song::Song>;

    fn new() -> Self;
}

pub enum ExternalType {
    LOCAL(LocalSong),
    OTHER,
}