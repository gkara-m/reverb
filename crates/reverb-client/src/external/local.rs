use anyhow::anyhow;
use audiotags::Tag;
use lofty::{file::AudioFile, probe::Probe};
use rodio::{Decoder, OutputStream, OutputStreamBuilder, Sink};
use serde::{Deserialize, Serialize};
use std::{fs::File, io::BufReader, path::Path, time::Duration, vec};

use crate::{
    external::external::{External, ExternalSong::LOCAL, ExternalSongTrait}, 
    internal::song::{Song, SongInfo},
    CONFIG,
};

use reverb_core::failure::failure::{Failure, FailureType};


pub struct Local {
    _output_stream: OutputStream,
    sink: Sink,
    song_duration: Duration,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub(crate) struct LocalSong {
    song_path: String,
    duration: Duration,
}

impl LocalSong {
    fn get_duration(&self) -> Duration {
        self.duration
    }
}

impl ExternalSongTrait for LocalSong {
    fn info(&self) -> Result<crate::internal::song::SongInfo, Failure> {
        let tag = Tag::new().read_from_path(&self.song_path);
        match tag {
            Ok(song_tag) => {
                let title = match song_tag.title() {
                    Some(title) => title.to_string(),
                    None => self.song_path.to_string(),
                };
                let artists = match song_tag.artists() {
                    Some(artists) => {
                        let mut artists_as_string: Vec<String> = Vec::new();
                        for artist in artists {
                            artists_as_string.push(artist.to_string());
                        }
                        artists_as_string
                    }
                    None => vec!["Unknown Artist".to_string()],
                };
                Ok(SongInfo { title, artists })
            }
            Err(_) => {
                return Ok(SongInfo {
                    title: self.song_path.to_string(),
                    artists: vec!["Unknown Artist".to_string()],
                });
            }
        }
    }

    fn new(path_str: &str) -> Result<Self, Failure> {
        let path = Path::new(&CONFIG.get().ok_or(Failure::from((anyhow!("CONFIG not set"), FailureType::Fatal)))?
            .local_song_folder_path).join(path_str);
        println!("Path: {}", path.to_string_lossy());

        let duration = {
            let tagged_file = Probe::open(&path)
                .map_err(|e| Failure::from((e.into(), "Failed to open path", FailureType::Warning)))?
                .read()
                .map_err(|e| Failure::from((e.into(), "Failed to read duration", FailureType::Warning)))?;
            tagged_file.properties().duration()
        };

        if path.exists() {
            Ok(LocalSong {
                song_path: path.to_string_lossy().to_string(),
                duration,
            })
        } else {
            Err(Failure::from((
                std::io::Error::new(std::io::ErrorKind::NotFound, "File does not exist").into(),
                FailureType::Warning,
            )))
        }
    }
}

impl External for Local {
    fn new(song: &Song) -> Result<Local, Failure> {
        let output_stream = match OutputStreamBuilder::open_default_stream() {
            Err(e) => return Err(Failure::from((e.into(), FailureType::Warning))),
            Ok(output_stream) => output_stream,
        };
        let sink = Sink::connect_new(&output_stream.mixer());
        sink.pause();
        let mut local = Local {
            _output_stream: output_stream,
            sink,
            song_duration: Duration::new(0, 0), // overwritten in load_new()
        };
        local.load_new(song)?;
        Ok(local)
    }

    fn play_new(&mut self, song: &Song) -> Result<(), Failure> {
        self.load_new(song)?;
        self.sink.play();
        Ok(())
    }

    fn pause(&self) -> Result<(), Failure> {
        self.sink.pause();
        Ok(())
    }

    fn play(&self) -> Result<(), Failure> {
        self.sink.play();
        Ok(())
    }

    fn stop(&self) -> Result<(), Failure> {
        self.sink.stop();
        Ok(())
    }

    fn shutdown(&self) -> Result<(), Failure> {
        // nothing to be done
        Ok(())
    }

    fn is_song_playing(&self) -> Result<bool, Failure> {
        Ok(!self.sink.is_paused())
    }

    fn song_duration_gone(&self) -> Result<Duration, Failure> {
        if self.sink.get_pos() >= self.song_duration {
            return Ok(self.song_duration);
        }
        Ok(self.sink.get_pos())
    }

    fn song_duration(&self) -> Result<Duration, Failure> {
        Ok(self.song_duration)
    }
}

impl Local {
    fn load_new(&mut self, song: &Song) -> Result<(), Failure> {
        if let LOCAL(ref local_song) = song.song_type {
            self.stop()?;
            let decoder = load_decoder(&local_song.song_path);
            self.song_duration = local_song.get_duration();
            self.sink.append(decoder);
            Ok(())
        } else {
            Err(Failure::from((
                anyhow!("Invalid song type for Local external"),
                FailureType::Warning,
            )))
        }
    }

    pub fn new(song: &Song) -> Result<Local, Failure> {
        let output_stream = match OutputStreamBuilder::open_default_stream() {
            Err(e) => return Err(Failure::from((e.into(), FailureType::Warning))),
            Ok(output_stream) => output_stream,
        };
        let sink = Sink::connect_new(&output_stream.mixer());
        sink.pause();
        let mut local = Local {
            _output_stream: output_stream,
            sink,
            song_duration: Duration::new(0, 0), // overwritten in load_new()
        };
        local.load_new(song)?;
        Ok(local)
    }
}

fn load_decoder(file_path: &str) -> Decoder<BufReader<File>> {
    Decoder::new(BufReader::new(File::open(file_path).unwrap())).unwrap()
}
