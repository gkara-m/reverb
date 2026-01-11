use std::fs::File;
use std::io::Write;

use serde::{Deserialize, Serialize};

use crate::{PLAYLIST_FOLDER, external::external::ExternalType, internal::{playlist, song::Song}};
#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct Playlist {
    name: String,
    songs: Vec<Song>,
    external_type: Option<ExternalType>,
}

impl Playlist {
    fn new(name: String, external_type: Option<ExternalType>) -> Playlist {
        Playlist {
            name,
            songs: Vec::new(),
            external_type,
        }
    }

    fn add(&mut self, song: Song) -> bool {
        self.songs.push(song);
        true
    }

    fn remove(&mut self, from: usize) -> bool {
        if from >= self.songs.len() {
            false
        } else {
            self.songs.remove(from);
            true
        }
    }

    fn get_songs(&self) -> &Vec<Song> {
        &self.songs
    }

    fn get_song(&self, index: usize) -> Option<&Song> {
        self.songs.get(index)
    }

    fn get_name(&self) -> &String {
        &self.name
    }

    fn move_song(&mut self, from: usize, to: usize) -> bool {
        if from >= self.songs.len() || to >= self.songs.len() {
            false
        } else {
            let song = self.songs.remove(from);
            self.songs.insert(to, song);
            true
        }
    }

    fn save(&self) -> Result<(), Box<dyn std::error::Error>> {
        let file = File::create(PLAYLIST_FOLDER.to_string() + &self.name + ".json")?;
        serde_json::to_writer_pretty(file, self)?;
        Ok(())
    }

    fn load(name: &str) -> Result<Playlist, Box<dyn std::error::Error>> {
        let file = File::open(PLAYLIST_FOLDER.to_string() + name + ".json")?;
        let playlist: Playlist = serde_json::from_reader(file)?;
        Ok(playlist)
    }
}