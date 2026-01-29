use serde::{Deserialize, Serialize};

use crate::{external::external::{External, ExternalSongTrait}, internal::song};


#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PlaceholderExternalSong;

pub struct PlaceholderRun;

impl ExternalSongTrait for PlaceholderExternalSong {
    fn info(&self) -> Result<crate::internal::song::SongInfo, String> {
        todo!()
    }
    
    fn new(_info: &str) -> Result<Self, String> where Self: Sized {
        todo!()
    }
}

impl External for PlaceholderRun {
    fn play_new(&self, _song: &song::Song) -> Result<(), String> {
        todo!()
    }

    fn pause(&self) -> Result<(), String> {
        todo!()
    }

    fn play(&self) -> Result<(), String> {
        todo!()
    }

    fn stop(&self) -> Result<(), String> {
        todo!()
    }

    fn shutdown(&self) -> Result<(), String> {
        todo!()
    }
    
    fn new(_song: &song::Song) -> Result<Self, String> where Self: Sized {
        todo!()
    }
}