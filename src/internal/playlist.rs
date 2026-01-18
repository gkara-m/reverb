use std::{fs::File, path::PathBuf};
use std::io::Write;

use serde::{Deserialize, Serialize};

use crate::{DATA_FOLDER, external::external::ExternalType, internal::{playlist, song::Song}};
#[derive(Serialize, Deserialize, Debug)]


pub(crate) struct Playlist {
    name: String,
    songs: Vec<Song>,
    external_type: Option<ExternalType>,
    playlist_folder: PathBuf,
}


impl Playlist {
    pub fn new(name: &str, external_type: Option<ExternalType>) -> Result<Playlist, String> {
        let dir_str = DATA_FOLDER.get().ok_or("DATA_FOLDER not set".to_string())?;
        let mut dir = PathBuf::from(dir_str);
        dir.push("playlists");
        std::fs::create_dir_all(&dir).map_err(|e| format!("Failed to create playlist directory: {}", e))?;
        Ok(Playlist {
            name: name.to_string(),
            songs: Vec::new(),
            external_type,
            playlist_folder: dir,
        })
    }

    pub fn add(&mut self, song: &Song) -> Result<(), String> {
        self.songs.push(song.clone());
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
        let path  = self.playlist_folder.join(format!("{}.json", &self.name));
        let file = File::create(path).map_err(|e| format!("Failed to create playlist file: {}", e))?;
        serde_json::to_writer_pretty(file, self).map_err(|e| format!("Failed to write to playlist file: {}", e))?;
        Ok(())
    }

    pub fn load(name: &str) -> Result<Playlist, String> {
        let dir_str = DATA_FOLDER.get().ok_or("DATA_FOLDER not set".to_string())?;
        let mut dir = PathBuf::from(dir_str);
        dir.push("playlists");
        let file = File::open(dir.join(format!("{}.json", name))).map_err(|e| format!("Failed to open playlist file: {}", e))?;
        serde_json::from_reader(file).map_err(|e| format!("Failed to parse playlist file: {}", e))
    }
}