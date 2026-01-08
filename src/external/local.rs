use std::fs::File;
use std::io::BufReader;
use rodio::{Decoder, OutputStream, source::Source, Sink, OutputStreamBuilder};

use crate::external::{external::{self, External, ExternalType::LOCAL}};
use crate::internal::song::Song;

pub struct Local{
    pub current_song: Option<Song>,
    output_stream: OutputStream,
    sink: Option<Sink>,
}

pub(crate) struct LocalSong {
    song_path : String,
}

impl LocalSong {
    pub fn new(song_path: &str) -> Self {
        LocalSong {
            song_path: song_path.to_string(),
        }
    }
}

impl External for Local {
    fn play_song(&mut self, song: &Song) -> bool {
        if let LOCAL(ref local_song) = song.song_type {
            let file = load_file(&local_song.song_path);
            let sink = play_file(&self.output_stream, file);
            self.sink = Some(sink);
            true
        } else {false}
    }

    fn pause(&self) -> bool {
        if let Some(ref sink) = self.sink {
            sink.pause();
            true
        } else {false}
    }

    fn play(&self) -> bool {
        if let Some(ref sink) = self.sink {
            sink.play();
            true
        } else {false}
    }

    fn stop(&mut self) -> bool {
        if let Some(sink) = self.sink.take() {
            sink.stop();
            self.sink = None;
            true
        } else {false}
    }

    fn new() -> Self {
        Local {
            current_song: None,
            output_stream: OutputStreamBuilder::open_default_stream()
            .expect("open default audio stream"),
            sink: None,
        }
    }
}

fn load_file(file_path: &str) -> BufReader<File> {
    BufReader::new(File::open(file_path).unwrap())
}

fn play_file(stream_handle: &OutputStream, file: BufReader<File>) -> Sink {
    rodio::play(&stream_handle.mixer(), file).unwrap()
}