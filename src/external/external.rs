use serde::{Deserialize, Serialize};

use crate::internal::song::Song;
use crate::external::local::{Local, LocalSong};

pub trait External {
    fn play_new(&self, song: &Song) -> Result<(), String>;

    fn pause(&self) -> Result<(), String>;

    fn play(&self) -> Result<(), String>;

    fn stop(&self) -> Result<(), String>;

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
    fn play_new(&self, song: &Song) -> Result<(), String> {
        self.as_external().play_new(song)
    }

    fn pause(&self) -> Result<(), String> {
        self.as_external().pause()
    }

    fn play(&self) -> Result<(), String> {
        self.as_external().play()
    }

    fn stop(&self) -> Result<(), String> {
        self.as_external().stop()
    }
}

/*
    Macro to generate ExternalRun, ExternalSong, and ExternalType enums,
    as well as implementations to create new ExternalSong and ExternalType instances from strings.

    will expand to something like:
    pub enum ExternalRun {
        LOCAL(Local),
        YOUTUBE(()),
    }
    pub enum ExternalSong {
        LOCAL(LocalSong),
        YOUTUBE(()),
    }
    pub enum ExternalType {
        LOCAL,
        YOUTUBE,
    }

    impl ExternalType {
        pub fn get_from_str(string: &str) -> Result<ExternalType, String> {
            match string {
                "local" => Ok(ExternalType::LOCAL),
                "youtube" => Ok(ExternalType::YOUTUBE),
                _ => Err(format!("Unknown external type: {}", string))
            }
        }
        
        pub fn new_external_song(&self, string: &str) -> Result<ExternalSong, String> {
            match self {
                ExternalType::LOCAL => LocalSong::new(string).map(ExternalSong::LOCAL),
                ExternalType::YOUTUBE => Ok(ExternalSong::YOUTUBE(())),
            }
        }
    }
*/


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

    #[derive(Clone, Debug, Serialize, Deserialize)]
    pub enum ExternalSong {
        $(
            $backend($song)
        ),*
    }

    #[derive(Clone, Debug, Serialize, Deserialize)]
    pub enum ExternalType {
        $(
            $backend
        ),*
    }

    impl ExternalType {
        pub fn get_from_str(string: &str) -> Result<ExternalType, String> {
            match string {
                $(
                    stringify!($name) => Ok(ExternalType::$backend),
                )*
                _ => Err(format!("Unknown external type: {}", string))
            }
        }
        
        pub fn new_external_song(&self, string: &str) -> Result<ExternalSong, String> {
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
        SongNew: |_: &str| Ok(()),
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

pub fn get_new_external_run_from_song(song: &Song) -> Result<ExternalRun, String> {
    match &song.song_type {
        ExternalSong::LOCAL(_) => Ok(ExternalRun::LOCAL(Local::new(song)?)),
        ExternalSong::YOUTUBE(_) => todo!(),
    }
}