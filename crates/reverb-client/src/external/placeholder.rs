use std::time::Duration;

use serde::{Deserialize, Serialize};

use crate::{external::external::{External, ExternalSongTrait}, failure::failure::Failure, internal::song};


#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PlaceholderExternalSong;

pub struct PlaceholderRun;

impl ExternalSongTrait for PlaceholderExternalSong {
    fn info(&self) -> Result<crate::internal::song::SongInfo, Failure> {
        todo!()
    }
    
    fn new(_info: &str) -> Result<Self, Failure> where Self: Sized {
        todo!()
    }
}

impl External for PlaceholderRun {
    fn play_new(&mut self, _song: &song::Song) -> Result<(), Failure> {
        todo!()
    }

    fn pause(&self) -> Result<(), Failure> {
        todo!()
    }

    fn play(&self) -> Result<(), Failure> {
        todo!()
    }

    fn stop(&self) -> Result<(), Failure> {
        todo!()
    }

    fn shutdown(&self) -> Result<(), Failure> {
        todo!()
    }
    
    fn new(_song: &song::Song) -> Result<Self, Failure> where Self: Sized {
        todo!()
    }
    
    fn is_song_playing(&self) -> Result<bool, Failure> {
        todo!()
    }
    
    fn song_duration_gone(&self) -> Result<std::time::Duration, Failure> {
        todo!()
    }

    fn song_duration(&self) -> Result<Duration, Failure> {
        todo!()
    }
}