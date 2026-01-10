use crate::internal::song::Song;
use crate::external::local::{Local, LocalSong};

pub trait External {
    fn play_new(&self, song: &Song) -> bool;

    fn pause(&self) -> bool;

    fn play(&self) -> bool;

    fn stop(&self) -> bool;

}

impl ExternalRun {
    pub fn as_external(&self) -> &dyn External {
        match self {
            ExternalRun::LOCAL(local) => local,
            ExternalRun::YOUTUBE(_) => todo!(),
        }
    }
}

impl External for ExternalRun {
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

macro_rules! external_types {
    (
        $(
            $backend:ident {
            Run:  $run:ty,
            Song: $song:ty $(,)?
        }
        ),* $(,)?
    ) => {

    pub enum ExternalRun {
        $(
            $backend($run)
        ),*
    }

    pub enum ExternalSong {
        $(
            $backend($song)
        ),*
    }

    pub enum ExternalType {
        $(
            $backend
        ),*
    }

    }
}

external_types!{
    LOCAL{
        Run: Local,
        Song: LocalSong
    },
    YOUTUBE{
        Run: (),
        Song: ()
    },
}

impl ExternalSong {
    pub fn same_type(&self, external_type: &ExternalRun) -> bool {
        match (self, external_type) {
            (ExternalSong::LOCAL(_), ExternalRun::LOCAL(_)) => true,
            (ExternalSong::YOUTUBE(_), ExternalRun::YOUTUBE(_)) => true,
            _ => false,
        }
    }
}

pub fn get_new_external_from_song(song: &Song) -> ExternalRun {
    match &song.song_type {
        ExternalSong::LOCAL(_) => ExternalRun::LOCAL(Local::new(song)),
        ExternalSong::YOUTUBE(_) => todo!(),
    }
}