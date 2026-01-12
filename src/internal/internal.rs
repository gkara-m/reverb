use crate::{external::external::{self, External, ExternalRun, ExternalType}, internal::{self, playlist::{self, Playlist}, song::Song}};
use std::collections::VecDeque;


pub struct Internal{
    current_external: ExternalRun,
    current_playlist: Playlist,
    queue: VecDeque<Song>,
}


impl  Internal {

    pub fn new(song: Song) -> Self {
        Internal {
            current_external: external::get_new_external_from_song(&song),
            current_playlist: Playlist::new("TODO: add playlist name as param", None),
            queue: VecDeque::from([song]),
        }
    }

    pub fn play(&self) -> bool {
        self.current_external.play()
    }

    pub fn pause(&self) -> bool {
        self.current_external.pause()
    }

    pub fn play_new(&mut self, song :Song) -> bool {
        self.stop();
        if !song.song_type.same_type(&self.current_external) {
            self.current_external = external::get_new_external_from_song(&song);
        }
        self.queue[0] = song;
        self.current_external.play_new(&self.queue[0])
    }

    fn stop(&self) -> bool {
        self.current_external.stop()
    }
}

impl Internal{
    pub fn load_playlist(&mut self, playlist_name: &str) -> bool {
        if (self.save_playlist()) == false {
            return false;
        }
        match Playlist::load(playlist_name) {
            Err(_) => return false,
            Ok(playlist) => {
                self.current_playlist = playlist
                //TODO: load into queue
            },
        }
        true
    }

    fn save_playlist(&self) -> bool {
        match self.current_playlist.save() {
            Ok(_) => true,
            Err(_) => false,
        }
    }

    pub fn new_playlist(&mut self, name: &str, external_type: Option<ExternalType>) -> bool{
        if (self.save_playlist()) == false {
            return false;
        }
        self.current_playlist = Playlist::new(name, external_type);
        true
    }

    pub fn playlist_add(&mut self, song: Song) -> bool{
        self.current_playlist.add(song)
    }

    pub fn playlist_remove(&mut self, index: usize) -> bool{
        self.current_playlist.remove(index)
    }

    pub fn playlist_move_song(&mut self, from: usize, to: usize) -> bool{
        self.current_playlist.move_song(from, to)
    }

    pub fn playlist_get_songs(&self) -> &Vec<Song>{
        self.current_playlist.get_songs()
    }
}

impl Internal{
    pub fn queue_add(&mut self, song: Song) -> bool {
        self.queue.push_back(song);
        true
    }

    pub fn queue_remove(&mut self, song_index: usize) -> bool {
        if self.queue.len() > 1 && song_index > 0 && song_index < self.queue.len() {
            self.queue.remove(song_index);
            return true;
        } 
        false
    }

    pub fn queue_list(&mut self) -> () {
        let mut count = 0;
        for song in &self.queue {
            println!("{count}: {} - {}", song.artist, song.title);
            count += 1;
        }
    }

    pub fn queue_next(&mut self) -> bool {
        if self.queue.len() > 1 {
            self.queue.pop_front();
            if let Some(next_song) = self.queue.front().cloned() {
                return self.play_new(next_song);
            }
        }
        false
    }

}