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

macro_rules! make_external_types {
    (
        $(
            $backend:ident {
            Run:  $run:ty,
            Song: $song:ty,
            SongNew: $song_new:expr,
            string_name: $name:ident $(,)?
        }
        ),* $(,)?
    ) => {

    pub enum ExternalRun {
        $(
            $backend($run)
        ),*
    }

    #[derive(Clone, Debug)]
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

    impl ExternalType {
        pub fn get_from_str(string: &str) -> Option<ExternalType> {
            match string {
                $(
                    stringify!($name) => Some(ExternalType::$backend),
                )*
                _ => None
            }
        }

        pub fn new_external_song(&self, string: &str) -> Option<ExternalSong> {
            match self {
                $(
                    ExternalType::$backend => $song_new(string).map(ExternalSong::$backend),
                )*
            }
        }
    }

    }
}

make_external_types!{
    LOCAL{
        Run: Local,
        Song: LocalSong,
        SongNew: LocalSong::new,
        string_name: local,
    },
    YOUTUBE{
        Run: (),
        Song: (),
        SongNew: |_: &str| Some(()),
        string_name: youtube,
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