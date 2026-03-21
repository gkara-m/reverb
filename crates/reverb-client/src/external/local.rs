use lofty::file::AudioFile;
use lofty::probe::Probe;
use rodio::{Decoder, OutputStream, OutputStreamBuilder, Sink};
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::BufReader;
use std::path::Path;
use std::time::Duration;
use anyhow::anyhow;

use crate::external::external::{External, ExternalSong::LOCAL, ExternalSongTrait};
use crate::failure::failure::{Failure, FailureType};
use crate::internal::song::Song;

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
        Ok(crate::internal::song::SongInfo {
            title: format!("Local Song at path: {}", self.song_path),
            artist: String::from("Unknown Artist"),
        })
    }

    fn new(path_str: &str) -> Result<Self, Failure> {
        let path = Path::new(path_str);
        let duration = {
            let tagged_file = Probe::open(&path)
                .map_err(|e| Failure::from((e.into(), FailureType::Warning)))?
                .read()
                .map_err(|e| Failure::from((e.into(), FailureType::Warning)))?;
            tagged_file.properties().duration()
        };

        if path.exists() {
            Ok(LocalSong {
                song_path: path_str.to_string(),
                duration,
            })
        } else {
            Err(Failure::from((std::io::Error::new(std::io::ErrorKind::NotFound, "File does not exist").into(), FailureType::Warning)))
        }
    }
}

impl External for Local {
    fn new(song:&Song) -> Result<Local, Failure> {
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
            Err(Failure::from((anyhow!("Invalid song type for Local external"), FailureType::Warning)))
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
