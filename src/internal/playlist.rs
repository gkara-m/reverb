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
    pub fn new(name: &str, external_type: Option<ExternalType>) -> Playlist {
        Playlist {
            name: name.to_string(),
            songs: Vec::new(),
            external_type,
        }
    }

    pub fn add(&mut self, song: Song) -> bool {
        self.songs.push(song);
        true
    }

    pub fn remove(&mut self, index: usize) -> bool {
        if index >= self.songs.len() {
            false
        } else {
            self.songs.remove(index);
            true
        }
    }

    pub fn get_songs(&self) -> &Vec<Song> {
        &self.songs
    }

    pub fn get_song(&self, index: usize) -> Option<&Song> {
        self.songs.get(index)
    }

    pub fn get_name(&self) -> &String {
        &self.name
    }

    pub fn move_song(&mut self, from: usize, to: usize) -> bool {
        if from >= self.songs.len() || to >= self.songs.len() {
            false
        } else {
            let song = self.songs.remove(from);
            self.songs.insert(to, song);
            true
        }
    }

    pub fn save(&self) -> Result<(), Box<dyn std::error::Error>> {
        std::fs::create_dir_all(PLAYLIST_FOLDER)?;
        let path = format!("{}{}.json", PLAYLIST_FOLDER, self.name);
        let file = File::create(path)?;
        serde_json::to_writer_pretty(file, self)?;
        Ok(())
    }

    pub fn load(name: &str) -> Result<Playlist, Box<dyn std::error::Error>> {
        let file = File::open(PLAYLIST_FOLDER.to_string() + name + ".json")?;
        let playlist: Playlist = serde_json::from_reader(file)?;
        Ok(playlist)
    }
}