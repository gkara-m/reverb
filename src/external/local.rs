use std::fs::File;
use std::io::BufReader;
use std::path::Path;
use rodio::{Decoder, OutputStream, Sink, OutputStreamBuilder};
use serde::{Deserialize, Serialize};

use crate::external::external::{External, ExternalSong::LOCAL, ExternalSongTrait};
use crate::internal::song::Song;


pub struct Local{
    _output_stream: OutputStream,
    sink: Sink,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub(crate) struct LocalSong {
    song_path : String,
}

impl ExternalSongTrait for LocalSong {
    fn info(&self) -> Result<crate::internal::song::SongInfo, String> {
        Ok(crate::internal::song::SongInfo {
            title: format!("Local Song at path: {}", self.song_path),
            artist: String::from("Unknown Artist"),
        })
    }

    fn new(path_str: &str) -> Result<Self, String> {
        let path = Path::new(path_str);
        
        if path.exists() {
            Ok(LocalSong { song_path: path_str.to_string() })
        } else {
            Err(format!("File path does not exist: {}", path_str))
        }
    }
}

impl External for Local {
    fn new(song:&Song) -> Result<Local, String> {
        let output_stream = match OutputStreamBuilder::open_default_stream() {
            Err(e) => return Err(format!("Failed to open output stream: {}", e)),
            Ok(output_stream) => output_stream,
        };
        let sink = Sink::connect_new(&output_stream.mixer());
        sink.pause();
        let local = Local {
            _output_stream: output_stream,
            sink,
        };
        local.load_new(song)?;
        Ok(local)
    }
    
    fn play_new(&self, song: &Song) -> Result<(), String> {
        self.load_new(song)?;
        self.sink.play();
        Ok(())
    }

    fn pause(&self) -> Result<(), String> {
        self.sink.pause();
        Ok(())
    }

    fn play(&self) -> Result<(), String> {
        self.sink.play();
        Ok(())
    }

    fn stop(&self) -> Result<(), String> {
        self.sink.stop();
        Ok(())
    }

    fn shutdown(&self) -> Result<(), String> {
        // nothing to be done
        Ok(())
    }
}

impl Local {
    fn load_new(&self, song: &Song) -> Result<(), String> {
        if let LOCAL(ref local_song) = song.song_type {
            self.stop()?;
            let decoder = load_decoder(&local_song.song_path);
            self.sink.append(decoder);
            Ok(())
        } else {Err(String::from("Invalid song type for Local external"))}
    }
}


fn load_decoder(file_path: &str) -> Decoder<BufReader<File>> {
    Decoder::new(BufReader::new(File::open(file_path).unwrap())).unwrap()
}
