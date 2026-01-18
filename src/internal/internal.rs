use crate::{external::external::{self, External, ExternalRun, ExternalType}, internal::{playlist::{Playlist}, queue::Queue, song::Song}};


pub struct Internal{
    current_external: ExternalRun,
    current_playlist: Playlist,
    queue: Queue,
}


impl  Internal {

    pub fn new(song: Song, playlist: Playlist) -> Result<Self, String> {
        Ok(Internal {
            current_external: external::get_new_external_run_from_song(&song)?,
            current_playlist: playlist,
            queue: Queue::new(song),
        })
    }

    pub fn play(&self) -> Result<(), String> {
        self.current_external.play()
    }

    pub fn pause(&self) -> Result<(), String> {
        self.current_external.pause()
    }

    pub fn play_new(&mut self, song :Song) -> Result<(), String> {
        self.stop()?;
        if !song.song_type.same_type(&self.current_external) {
            self.current_external = external::get_new_external_run_from_song(&song)?;
        }
        self.queue.queued_songs[0] = song;
        self.current_external.play_new(&self.queue.queued_songs[0])
    }

    fn stop(&self) -> Result<(), String> {
        self.current_external.stop()
    }
}

impl Internal{
    pub fn load_playlist(&mut self, playlist_name: &str) -> Result<(), String> {
        self.save_playlist()?;
        let playlist = Playlist::load(playlist_name)?;
        self.current_playlist = playlist;
        Ok(())
    }

    fn save_playlist(&self) -> Result<(), String> {
        self.current_playlist.save()
    }

    pub fn new_playlist(&mut self, name: &str, external_type: Option<ExternalType>) -> Result<(), String>{
        self.save_playlist()?;
        self.current_playlist = Playlist::new(name, external_type)?;
        Ok(())
    }

    pub fn playlist_add(&mut self, song: Song) -> Result<(), String>{
        self.current_playlist.add(&song)
    }

    pub fn playlist_remove(&mut self, index: usize) -> Result<(), String>{
        self.current_playlist.remove(index)
    }

    pub fn playlist_move_song(&mut self, from: usize, to: usize) -> Result<(), String>{
        self.current_playlist.move_song(from, to)
    }

    pub fn playlist_get_songs(&self) -> Result<&Vec<Song>, String>{
        self.current_playlist.get_songs()
    }

}

impl Internal{

    pub fn queue_add(&mut self, song: Song) -> Result<(), String> {
        self.queue.add(song);
        Ok(())
    }

    pub fn queue_remove(&mut self, song_index: usize) -> Result<(), String> {
        self.queue.remove(song_index);
        Ok(())
    }

    pub fn queue_list(&mut self) -> Result<(), String> {
        self.queue.list();
        Ok(())
    }

    pub fn queue_next(&mut self) -> Result<(), String> {
        let next_song = self.queue.next()?;
        if let song = &next_song {
            self.play_new(next_song);
        }
        Ok(())
    }
}