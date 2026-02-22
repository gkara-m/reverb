use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::{DATA_FOLDER, external::external::ExternalType, internal::song::Song};



#[derive(Serialize, Deserialize, Debug)]
pub struct Playlist {
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
        let playlist = Playlist {
            name: name.to_string(),
            songs: Vec::new(),
            external_type,
            playlist_folder: dir,
        };
        playlist.save()?;
        Ok(playlist)
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

    pub fn get_songs(&self) -> Result<Vec<Song>, String> {
        Ok(self.songs.clone())
    }

    pub fn get_song(&self, index: usize) -> Result<Song, String> {
        match self.songs.get(index) {
            Some(song) => Ok(song.clone()),
            None => Err(format!("invalid song index: {}", index)),
        }
    }

    pub fn get_name(&self) -> Result<String, String> {
        Ok(self.name.clone())
    }

    pub fn set_name(&mut self, name: &str) -> Result<(), String> {
        let new_path = self.playlist_folder.join(format!("{}.json", name));
        if new_path.exists() {
            return Err(format!("a playlist with the name '{}' already exists \n use playlist load {} to load it", name, name));
        }
        let old_path = self.playlist_folder.join(format!("{}.json", &self.name));
        if old_path.exists() {
            std::fs::rename(&old_path, &new_path).map_err(|e| format!("failed to rename playlist file: {}", e))?;
        }
        self.name = name.to_string();
        Ok(())
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
        match std::fs::write(
            self.playlist_folder.join(format!("{}.toml", &self.name)),
            toml::to_string(self).map_err(|e| format!("Failed to serialize startup data: {}", e))?,
        ) {
            Err(e) => Err(format!("Failed to write startup file: {}", e)),
            Ok(_) => Ok(()),
        }
    }

    pub fn load(name: &str) -> Result<Playlist, String> {
        let dir_str = DATA_FOLDER.get().ok_or("DATA_FOLDER not set".to_string())?;
        let mut dir = PathBuf::from(dir_str);
        dir.push("playlists");

        Ok(toml::from_str(
            &std::fs::read_to_string(dir.join(format!("{}.toml", name)))
            .map_err(|e| format!("Failed to read playlist data: {}", e))?
        ).map_err(|e| format!("Failed to parse playlist data: {}", e))?)
    }

    pub fn iter(&self) -> std::slice::Iter<'_, Song> {
        self.songs.iter()
    } 
}