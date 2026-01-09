use crate::{external::{external::{self, External, ExternalType}}, internal::song::Song};


pub struct Internal{
    current_external: ExternalType,
    current_song: Song,
}


impl  Internal {

    pub fn new(song: Song) -> Self {
        Internal {
            current_external: external::get_new_external_type_from_song(&song),
            current_song: song,
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
            self.current_external = external::get_new_external_type_from_song(&song);
        }
        self.current_song = song;
        self.current_external.play_new(&self.current_song)
    }

    fn stop(&self) -> bool {
        self.current_external.stop()
    }

}