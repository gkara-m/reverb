use crate::{external::{external::{self, External, ExternalRun}}, internal::song::Song};
use std::collections::VecDeque;


pub struct Internal{
    current_external: ExternalRun,
    queue: VecDeque<Song>,
}


impl  Internal {

    pub fn new(song: Song) -> Self {
        Internal {
            current_external: external::get_new_external_from_song(&song),
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

    pub fn queue_add(&mut self, song: Song) -> () {}

    pub fn queue_remove(&mut self, song: Song) -> () {}

    pub fn queue_next(&mut self, song: Song) -> () {}

}