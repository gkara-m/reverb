use std::collections::VecDeque;
use crate::internal::{playlist::Playlist, song::Song};


pub(super) struct Queue {
    pub(super) queued_songs: VecDeque<Song>
}


impl Queue {

    pub fn new(song: Song) -> Queue {
        Queue {queued_songs: VecDeque::from([song])}
    }

    pub fn add(&mut self, song: Song) -> Result<(), String> {
        self.queued_songs.push_back(song);
        Ok(())
    }

    pub fn load_playlist(&mut self, playlist: &Playlist) -> Result<(), String> {
        for song in playlist.get_songs()?.clone().into_iter().rev() {
            self.queued_songs.push_front(song);
        }
        Ok(())
    }

    pub fn remove(&mut self, song_index: usize) -> Result<(), String> {
        if self.queued_songs.len() <= 1 { return Err(String::from("Cannot remove song from queue: only one song in queue")); }
        if song_index <= 0 { return Err(String::from("Cannot remove currently playing song from queue")); }
        if song_index >= self.queued_songs.len() { return Err(format!("Invalid song index (too large): {}", song_index)); }
        self.queued_songs.remove(song_index);
        Ok(())
    }

    pub fn list(&mut self) -> Result<(), String> {
        for song in self.queued_songs.iter() {
            println!("{} - {}", song.artist, song.title);
        }
        Ok(())
    }

    pub fn next(&mut self) -> Result<Song, String> {
        if self.queued_songs.len() > 1 {
            self.queued_songs.pop_front();
            if let Some(next_song) = self.queued_songs.front().cloned() {
                return Ok(next_song);
            }
            return Err(String::from("Queue is empty"))
        }
        Err(String::from("Queue has only one song or is empty"))
    }

    pub fn current_song(&self) -> Result<Song, String> {
        self.queued_songs.get(0)
            .cloned()
            .ok_or_else(|| String::from("Queue is empty"))
    }
}