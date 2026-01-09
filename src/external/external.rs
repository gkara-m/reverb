use crate::internal::song::Song;
use crate::external::local::{Local, LocalSong};

pub trait External {
    fn play_new(&self, song: &Song) -> bool;

    fn pause(&self) -> bool;

    fn play(&self) -> bool;

    fn stop(&self) -> bool;

}

impl ExternalType {
    pub fn as_external(&self) -> &dyn External {
        match self {
            ExternalType::LOCAL(local) => local,
            ExternalType::YOUTUBE() => todo!(),
        }
    }
}

impl External for ExternalType {
    fn play_new(&self, song: &Song) -> bool {
        self.as_external().play_new(song)
    }

    fn pause(&self) -> bool {
        self.as_external().pause()
    }

    fn play(&self) -> bool {
        self.as_external().play()
    }

    fn stop(&self) -> bool {
        self.as_external().stop()
    }
}

pub enum ExternalType {
    LOCAL(Local),
    YOUTUBE(),
}
pub enum ExternalSong {
    LOCAL(LocalSong),
    YOUTUBE(String),
}

impl ExternalSong {
    pub fn same_type(&self, external_type: &ExternalType) -> bool {
        match (self, external_type) {
            (ExternalSong::LOCAL(_), ExternalType::LOCAL(_)) => true,
            (ExternalSong::YOUTUBE(_), ExternalType::YOUTUBE()) => true,
            _ => false,
        }
    }
}

pub fn get_new_external_from_song(song: &Song) -> ExternalType {
    match &song.song_type {
        ExternalSong::LOCAL(_) => ExternalType::LOCAL(Local::new(song)),
        ExternalSong::YOUTUBE(_) => todo!(),
    }
}