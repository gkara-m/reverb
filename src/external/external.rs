use std::time::Duration;

use serde::{Deserialize, Serialize};
use anyhow::anyhow;

use crate::external::local::{Local, LocalSong};
use crate::external::placeholder::{PlaceholderExternalSong, PlaceholderRun};
use crate::Song;
use crate::failure::failure::{Failure, FailureType};
pub trait External {
    fn play_new(&mut self, song: &Song) -> Result<(), Failure>;

    fn pause(&self) -> Result<(), Failure>;

    fn play(&self) -> Result<(), Failure>;

    fn stop(&self) -> Result<(), Failure>;

    fn shutdown(&self) -> Result<(), Failure>;

    fn new(song: &Song) -> Result<Self, Failure> where Self: Sized;

    fn is_song_playing(&self) -> Result<bool, Failure>;

    fn song_duration_gone(&self) -> Result<Duration, Failure>;

    fn song_duration(&self) -> Result<Duration, Failure>;
}

pub trait ExternalSongTrait {
    fn new(info: &str) -> Result<Self, Failure> where Self: Sized;
    fn info(&self) -> Result<crate::internal::song::SongInfo, Failure>;
}


impl External for ExternalRun {
    fn new(song: &Song) -> Result<Self, Failure> where Self: Sized {
        get_new_external_run_from_song(song)
    }

    fn play_new(&mut self, song: &Song) -> Result<(), Failure> {
        self.as_external_mut().play_new(song)
    }

    fn pause(&self) -> Result<(), Failure> {
        self.as_external().pause()
    }

    fn play(&self) -> Result<(), Failure> {
        self.as_external().play()
    }

    fn stop(&self) -> Result<(), Failure> {
        self.as_external().stop()
    }

    fn shutdown(&self) -> Result<(), Failure> {
        self.as_external().shutdown()
    }

    fn is_song_playing(&self) -> Result<bool, Failure> {
        self.as_external().is_song_playing()
    }

    fn song_duration_gone(&self) -> Result<Duration, Failure> {
        self.as_external().song_duration_gone()
    }

    fn song_duration(&self) -> Result<Duration, Failure> {
        self.as_external().song_duration()
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
        pub fn get_from_str(string: &str) -> Result<ExternalType, Failure> {
            match string {
                "local" => Ok(ExternalType::LOCAL),
                "youtube" => Ok(ExternalType::YOUTUBE),
                _ => Err(format!("Unknown external type: {}", string))
            }
        }

        pub fn new_external_song(&self, string: &str) -> Result<ExternalSong, Failure> {
            match self {
                ExternalType::LOCAL => LocalSong::new(string).map(ExternalSong::LOCAL),
                ExternalType::YOUTUBE => Ok(ExternalSong::YOUTUBE(())),
                }
            }
        }
    }

    impl ExternalSongTrait for ExternalSong {
        fn new(_info: &str) -> Result<Self, Failure> where Self: Sized {
            Err(Failure::from(("Use ExternalType::new_external_song instead", FailureType::Warning)))
        }
        fn info(&self) -> Result<crate::internal::song::SongInfo, Failure> {
            match self {
                ExternalSong::LOCAL(song) => song.info(),
                ExternalSong::YOUTUBE(_) => Err("No info available for YouTube placeholder".to_string()),
            }
        }
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

    pub fn get_new_external_run_from_song(song: &Song) -> Result<ExternalRun, Failure> {
        match &song.song_type {
            ExternalSong::LOCAL(_) => Ok(ExternalRun::LOCAL(Local::new(song)?)),
            ExternalSong::YOUTUBE(_) => todo!(),
        }
    }
    
    impl ExternalRun {
        pub fn as_external(&self) -> &dyn External {
            match self {
                ExternalRun::LOCAL(local) => local,
                ExternalRun::YOUTUBE(_) => todo!(),
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
            pub fn get_from_str(string: &str) -> Result<ExternalType, Failure> {
                match string {
                    $(
                        stringify!($name) => Ok(ExternalType::$backend),
                    )*
                    _ => Err(Failure::from((anyhow!("Unknown external type: {}", string), FailureType::Warning)))
                }
            }
            
            pub fn new_external_song(&self, string: &str) -> Result<ExternalSong, Failure> {
                match self {
                    $(
                        ExternalType::$backend => <$song>::new(string).map(ExternalSong::$backend),
                    )*
                }
            }
        }

        impl ExternalSongTrait for ExternalSong {
            // Note: This function is not intended to be used; it's here to satisfy the trait requirement.
            // Use ExternalType::new_external_song instead.
            // TODO: this will be chnaged later to remove confusion
            fn new(_info: &str) -> Result<Self, Failure> where Self: Sized {
                Err(Failure::from((anyhow!("Use ExternalType::new_external_song instead"), FailureType::Warning)))
            }
            fn info(&self) -> Result<crate::internal::song::SongInfo, Failure> {
                match self {
                    $(
                        ExternalSong::$backend(song) => song.info(),
                    )*
                }
            }
        }


        impl ExternalSong {
            pub fn same_type(&self, external_type: &ExternalRun) -> bool {
                match (self, external_type) {
                    $(
                        (ExternalSong::$backend(_), ExternalRun::$backend(_)) => true,
                    )*
                    _ => false,
                }
            }
        }

        pub fn get_new_external_run_from_song(song: &Song) -> Result<ExternalRun, Failure> {
            match &song.song_type {
                $(
                    ExternalSong::$backend(_) => Ok(ExternalRun::$backend(<$run>::new(song)?)),
                )*
            }
        }

        impl ExternalRun {
            pub fn as_external(&self) -> &dyn External {
                match self {
                    $(
                        ExternalRun::$backend(instance) => instance,
                    )*
                }
            }

            pub fn as_external_mut(&mut self) -> &mut dyn External {
                match self {
                    $(
                        ExternalRun::$backend(instance) => instance,
                    )*
                }
            }
        }
    }
}

// Song must implement ExternalSongInfo and 
// Run must implement External and NewExternal
make_external_types! {
    LOCAL{
        Run: Local,
        Song: LocalSong,
        string_name: local,
    },
    YOUTUBE{
        Run: PlaceholderRun,
        Song: PlaceholderExternalSong,
        string_name: youtube,
    },
}