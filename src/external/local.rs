use rodio::{Decoder, OutputStream, OutputStreamBuilder, Sink, Source, decoder};
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::BufReader;
use std::path::Path;
use std::time::Duration;

use crate::external::external::{External, ExternalSong::LOCAL};
use crate::internal::song::Song;

pub struct Local {
    _output_stream: OutputStream,
    sink: Sink,
    song_duration: Duration,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub(crate) struct LocalSong {
    song_path: String,
}

impl LocalSong {
    pub fn new(path_str: &str) -> Result<Self, String> {
        let path = Path::new(path_str);

        if path.exists() {
            Ok(LocalSong {
                song_path: path_str.to_string(),
            })
        } else {
            Err(format!("File path does not exist: {}", path_str))
        }
    }
}

impl External for Local {
    fn play_new(&mut self, song: &Song) -> Result<(), String> {
        self.load_new(song)?;
        self.sink.play();
        Ok(())
    }

    fn pause(&mut self) -> Result<(), String> {
        self.sink.pause();
        Ok(())
    }

    fn play(&mut self) -> Result<(), String> {
        self.sink.play();
        Ok(())
    }

    fn stop(&mut self) -> Result<(), String> {
        self.sink.stop();
        Ok(())
    }

    fn shutdown(&mut self) -> Result<(), String> {
        // nothing to be done
        Ok(())
    }

    fn sleep_until_song_end(&mut self) -> Result<(), String> {
        self.sink.sleep_until_end();
        Ok(())
    }

    fn is_song_playing(&mut self) -> Result<bool, String> {
        Ok(!self.sink.is_paused())
    }

    fn time_left(&mut self) -> Result<Duration, String> {
        Ok(self.song_duration - self.sink.get_pos())
    }
}

impl Local {
    fn load_new(&mut self, song: &Song) -> Result<(), String> {
        if let LOCAL(ref local_song) = song.song_type {
            self.stop()?;
            let decoder = load_decoder(&local_song.song_path);
            self.song_duration = decoder
                .total_duration()
                .ok_or(format!("placeholder error"))?;
            self.sink.append(decoder);
            Ok(())
        } else {
            Err(String::from("Invalid song type for Local external"))
        }
    }

    pub fn new(song: &Song) -> Result<Local, String> {
        let output_stream = match OutputStreamBuilder::open_default_stream() {
            Err(e) => return Err(format!("Failed to open output stream: {}", e)),
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
