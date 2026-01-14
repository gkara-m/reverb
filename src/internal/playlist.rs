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

    pub fn add(&mut self, song: Song) -> Result<(), String> {
        self.songs.push(song);
        Ok(())
    }

    pub fn remove(&mut self, index: usize) -> Result<(), String> {
        if index >= self.songs.len() {
            Err(format!("invalid song index: {}", index))
        } else {
            self.songs.remove(index);
            Ok(())
        }
    }

    pub fn get_songs(&self) -> Result<&Vec<Song>, String> {
        Ok(&self.songs)
    }

    pub fn get_song(&self, index: usize) -> Result<&Song, String> {
        match self.songs.get(index) {
            Some(song) => Ok(song),
            None => Err(format!("invalid song index: {}", index)),
        }
    }

    pub fn get_name(&self) -> Result<&String, String> {
        Ok(&self.name)
    }

    pub fn move_song(&mut self, from: usize, to: usize) -> Result<(), String> {
        if from >= self.songs.len() || to >= self.songs.len() {
            Err(format!("invalid song indices: from {}, to {}", from, to))
        } else {
            let song = self.songs.remove(from);
            self.songs.insert(to, song);
            Ok(())
        }
    }

    pub fn save(&self) -> Result<(), String> {
        match std::fs::create_dir_all(PLAYLIST_FOLDER) {
            Err(e) => return Err(format!("Failed to create playlist directory: {}", e)),
            Ok(_) => {}
        }
        let path = format!("{}{}.json", PLAYLIST_FOLDER, self.name);
        match File::create(path) {
            Err(e) => return Err(format!("Failed to create playlist file: {}", e)),
            Ok(file) => {
                match serde_json::to_writer_pretty(file, self) {
                    Err(e) => return Err(format!("Failed to write to playlist file: {}", e)),
                    Ok(_) => {}
                }
            }
        }
        Ok(())
    }

    pub fn load(name: &str) -> Result<Playlist, String> {
        match File::open(PLAYLIST_FOLDER.to_string() + name + ".json") {
            Err(e) => return Err(format!("Failed to open playlist file: {}", e)),
            Ok(file) => {
                match serde_json::from_reader(file) {
                    Err(e) => return Err(format!("Failed to parse playlist file: {}", e)),
                    Ok(playlist) => return Ok(playlist),
                }
            }
        }
    }
}