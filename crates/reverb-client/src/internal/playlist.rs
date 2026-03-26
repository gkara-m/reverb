use std::path::PathBuf;
use anyhow::anyhow;
use serde::{Deserialize, Serialize};

use crate::{DATA_FOLDER, external::external::ExternalType, failure::failure::{Failure, FailureType}, internal::song::Song};

#[derive(Serialize, Deserialize, Debug)]
pub struct Playlist {
    name: String,
    songs: Vec<Song>,
    external_type: Option<ExternalType>,
}


impl Playlist {
    pub fn new(name: &str, external_type: Option<ExternalType>) -> Result<Playlist, Failure> {
        let dir_str = DATA_FOLDER.get().ok_or(Failure::from((anyhow!("DATA_FOLDER not set"), FailureType::Fatal)))?;
        let mut dir = PathBuf::from(dir_str);
        dir.push("playlists");
        if dir.join(format!("{}.toml", name)).exists() {
            return Err(Failure::from((anyhow!("A playlist with the name '{}' already exists", name), FailureType::Warning)));
        }
        std::fs::create_dir_all(&dir).map_err(|e| Failure::from((e.into(), FailureType::Warning)))?;
        let playlist = Playlist {
            name: name.to_string(),
            songs: Vec::new(),
            external_type,
        };
        playlist.save()?;
        Ok(playlist)
    }

    pub(super) fn add(&mut self, song: &Song) {
        self.songs.push(song.clone());
    }

    pub(super) fn remove(&mut self, index: usize) -> Result<(), Failure> {
        if index >= self.songs.len() {
            Err(Failure::from((anyhow!("invalid song index: {}", index), FailureType::Warning)))
        } else {
            self.songs.remove(index);
            Ok(())
        }
    }

    pub(super) fn clear(&mut self) {
        self.songs.clear();
    }

    pub(super) fn get_songs(&self) -> Vec<Song> {
        self.songs.clone()
    }

    pub fn get_song(&self, index: usize) -> Result<Song, Failure> {
        match self.songs.get(index) {
            Some(song) => Ok(song.clone()),
            None => Err(Failure::from((anyhow!("invalid song index: {}", index), FailureType::Warning))),
        }
    }

    pub(super) fn get_name(&self) -> String {
        self.name.clone()
    }

    pub(super) fn set_name(&mut self, name: &str) -> Result<(), Failure> {
        let dir_str = DATA_FOLDER.get().ok_or(Failure::from((anyhow!("DATA_FOLDER not set"), FailureType::Fatal)))?;
        let mut dir = PathBuf::from(dir_str);
        dir.push("playlists");
        let new_path = dir.join(format!("{}.toml", name));
        if new_path.exists() {
            return Err(Failure::from((anyhow!("A playlist with the name '{}' already exists", name), FailureType::Warning)));
        }
        let old_path = dir.join(format!("{}.toml", &self.name));
        if old_path.exists() {
            std::fs::rename(&old_path, &new_path).map_err(|e| Failure::from((e.into(), FailureType::Warning)))?;
        }
        self.name = name.to_string();
        Ok(())
    }

    pub(super) fn move_song(&mut self, from: usize, to: usize) -> Result<(), Failure> {
        if from >= self.songs.len() || to >= self.songs.len() {
            Err(Failure::from((anyhow!("invalid song indices: from {}, to {}", from, to), FailureType::Warning)))
        } else {
            let song = self.songs.remove(from);
            self.songs.insert(to, song);
            Ok(())
        }
    }

    pub(super) fn save(&self) -> Result<(), Failure> {
        let dir_str = DATA_FOLDER.get().ok_or(Failure::from((anyhow!("DATA_FOLDER not set"), FailureType::Fatal)))?;
        let mut dir = PathBuf::from(dir_str);
        dir.push("playlists");
        match std::fs::write(
            dir.join(format!("{}.toml", &self.name)),
            toml::to_string(self).map_err(|e| Failure::from((e.into(), FailureType::Warning)))?,
        ) {
            Err(e) => Err(Failure::from((e.into(), FailureType::Warning))),
            Ok(_) => Ok(()),
        }
    }

    pub fn load(name: &str) -> Result<Playlist, Failure> {
        let dir_str = DATA_FOLDER.get().ok_or(Failure::from((anyhow!("DATA_FOLDER not set"), FailureType::Fatal)))?;
        let mut dir = PathBuf::from(dir_str);
        dir.push("playlists");

        Ok(toml::from_str(
            &std::fs::read_to_string(dir.join(format!("{}.toml", name)))
            .map_err(|e| Failure::from((e.into(), "playlist_load", FailureType::Warning)))?
        ).map_err(|e| Failure::from((e.into(), "playlist_load, playlist does not exist", FailureType::Warning)))?)
    }

    pub(super) fn iter(&self) -> std::slice::Iter<'_, Song> {
        self.songs.iter()
    }

    pub(super) fn get_type(&self) -> Option<ExternalType> {
        self.external_type.clone()
    }
}
