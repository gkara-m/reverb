use crate::{external::external::ExternalType, internal::song::Song};

pub(crate) struct Playlist {
    name: String,
    songs: Vec<Song>,
    external_type: Option<ExternalType>,
}

impl Playlist {
    fn new(name: String, external_type: Option<ExternalType>) -> Playlist {
        Playlist {
            name,
            songs: Vec::new(),
            external_type,
        }
    }

    fn add(&mut self, song: Song) -> bool {
        self.songs.push(song);
        true
    }

    fn remove(&mut self, from: usize) -> bool {
        if from >= self.songs.len() {
            false
        } else {
            self.songs.remove(from);
            true
        }
    }

    fn get_songs(&self) -> &Vec<Song> {
        &self.songs
    }

    fn get_song(&self, index: usize) -> Option<&Song> {
        self.songs.get(index)
    }

    fn get_name(&self) -> &String {
        &self.name
    }

    fn move_song(&mut self, from: usize, to: usize) -> bool {
        todo!()
    }
}