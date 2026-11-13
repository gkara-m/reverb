use std::{fs::File, path::PathBuf};
use anyhow::anyhow;
use serde::{Deserialize, Serialize};

use crate::{config::config::config, external::external::ExternalType, internal::{playlist::m3u::{file_to_m3u, m3u_to_playlist, playlist_to_m3u_string}, song::Song}};
use reverb_core::failure::failure::{Failure, FailureType};


#[derive(Serialize, Deserialize, Debug)]
pub struct Playlist {
    pub(super) name: String,
    pub(super) songs: Vec<Song>,
    pub(super) external_type: Option<ExternalType>,
}
    

impl Playlist {
    pub fn new(name: &str, external_type: Option<ExternalType>) -> Result<Playlist, Failure> {
        let dir_str = config()?.data_folder.clone();
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

    pub fn add(&mut self, song: &Song) {
        self.songs.push(song.clone());
    }

    pub fn remove(&mut self, index: usize) -> Result<(), Failure> {
        if index >= self.songs.len() {
            Err(Failure::from((anyhow!("invalid song index: {}", index), FailureType::Warning)))
        } else {
            self.songs.remove(index);
            Ok(())
        }
    }

    pub fn clear(&mut self) {
        self.songs.clear();
    }

    pub fn get_songs(&self) -> Vec<Song> {
        self.songs.clone()
    }

    pub fn get_song(&self, index: usize) -> Result<Song, Failure> {
        match self.songs.get(index) {
            Some(song) => Ok(song.clone()),
            None => Err(Failure::from((anyhow!("invalid song index: {}", index), FailureType::Warning))),
        }
    }

    pub fn get_name(&self) -> String {
        self.name.clone()
    }

    pub fn set_name(&mut self, name: &str) -> Result<(), Failure> {
        let dir_str = config()?.data_folder.clone();
        let mut dir = PathBuf::from(dir_str);
        dir.push("playlists");
        let new_path = dir.join(format!("{}.m3u", name));
        if new_path.exists() {
            return Err(Failure::from((anyhow!("A playlist with the name '{}' already exists", name), FailureType::Warning)));
        }
        let old_path = dir.join(format!("{}.m3u", &self.name));
        if old_path.exists() {
            std::fs::rename(&old_path, &new_path).map_err(|e| Failure::from((e.into(), FailureType::Warning)))?;
        }
        self.name = name.to_string();
        Ok(())
    }

    pub fn move_song(&mut self, from: usize, to: usize) -> Result<(), Failure> {
        if from >= self.songs.len() || to >= self.songs.len() {
            Err(Failure::from((anyhow!("invalid song indices: from {}, to {}", from, to), FailureType::Warning)))
        } else {
            let song = self.songs.remove(from);
            self.songs.insert(to, song);
            Ok(())
        }
    }

    pub fn save(&self) -> Result<(), Failure> {
        let dir_str = config()?.data_folder.clone();
        let mut dir = PathBuf::from(dir_str);
        dir.push("playlists");
        match std::fs::write(
            dir.join(format!("{}.m3u", &self.name)),
            playlist_to_m3u_string(self)?,
        ) {
            Err(e) => Err(Failure::from((e.into(), FailureType::Warning))),
            Ok(_) => Ok(()),
        }
    }

    pub fn load(name: &str) -> Result<Playlist, Failure> {
        println!("Loading playlist: {}", name);
        let dir_str = config()?.data_folder.clone();
        println!("Data folder: {}", dir_str);
        let mut dir = PathBuf::from(dir_str);
        println!("Playlist directory: {:?}", dir);
        dir.push("playlists");
        println!("Full playlist path: {:?}", dir.join(format!("{}.m3u", name)));

        let m3u = file_to_m3u(
            File::open(dir.join(format!("{}.m3u", name)))
            .map_err(|e| Failure::from((e.into(), "playlist_load", FailureType::Warning)))?
        )?;
        m3u_to_playlist(m3u)
    }

    pub fn iter(&self) -> std::slice::Iter<'_, Song> {
        self.songs.iter()
    }

    pub fn get_type(&self) -> Option<ExternalType> {
        self.external_type.clone()
    }
}