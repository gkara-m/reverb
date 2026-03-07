use std::collections::VecDeque;
use anyhow::anyhow;

use serde::{Deserialize, Serialize};

use crate::{failure::failure::{Failure, FailureType}, internal::{playlist::Playlist, song::Song}};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Queue {
    pub(super) queued_songs: VecDeque<Song>
}


impl Queue {

    pub fn new(song: Song) -> Queue {
        Queue {queued_songs: VecDeque::from([song])}
    }

    pub fn add(&mut self, song: Song) {
        self.queued_songs.push_back(song);
    }

    pub fn load_playlist(&mut self, playlist: &Playlist) {
        for song in playlist.iter() {
            self.queued_songs.push_front(song.clone());
        }
    }

    pub fn remove(&mut self, song_index: usize) -> Result<(), Failure> {
        if self.queued_songs.len() <= 1 { return Err(Failure::from((anyhow!("Cannot remove song from queue: only one song in queue"), FailureType::Warning))); }
        if song_index == 0 { return Err(Failure::from((anyhow!("Cannot remove currently playing song from queue at index 0"), FailureType::Warning))); }
        if song_index >= self.queued_songs.len() { return Err(Failure::from((anyhow!("Invalid song index (too large): {}", song_index), FailureType::Warning))); }
        self.queued_songs.remove(song_index);
        Ok(())
    }

    pub fn get_songs(&self) -> Vec<Song> {
        self.iter().collect()
    }

    pub fn next(&mut self) -> Result<Song, Failure> {
        if self.queued_songs.len() > 1 {
            self.queued_songs.pop_front();
            match self.queued_songs.front().cloned() {
                Some(next_song) => return Ok(next_song),
                None => return Err(Failure::from((anyhow!("Queue is empty after popping current song, queue should not be able to pop only song"), FailureType::Fetal))),}
        }
        Err(Failure::from((anyhow!("Queue has only one song or is empty"), FailureType::Warning)))
    }

    pub fn current_song(&self) -> Result<Song, Failure> {
        self.queued_songs.get(0)
            .cloned()
            .ok_or_else(|| Failure::from((anyhow!("Queue is empty"), FailureType::Warning)))
    }

    pub fn clear(&mut self) {
        let current_song = self.queued_songs.pop_front();
        self.queued_songs.clear();
        match current_song {
            Some(song) => self.queued_songs.push_back(song),
            None => {unreachable!("Queue should always have at least one song when clear is called please report this bug")},
        }
    }

    pub fn iter(&self) -> impl Iterator<Item = Song> {
        self.queued_songs.iter().cloned()
    }
}