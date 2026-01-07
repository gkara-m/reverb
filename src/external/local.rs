use std::fs::File;
use std::io::BufReader;
use rodio::{Decoder, OutputStream, source::Source, Sink};

use crate::external::{external, local};
use crate::internal::song;

pub struct Local{
    pub current_song: Option<song::Song>,
    output_stream: OutputStream,
}

pub(crate) struct LocalSong {
    curr_song_path : String,
    playing: Playing,
}

impl LocalSong {
    pub fn new(song_path: &str) -> Self {
        LocalSong {
            curr_song_path: song_path.to_string(),
            playing: Playing::Stopped,
        }
    }
}

pub enum Playing {
    Playing(Sink),
    Paused(Sink),
    Stopped,
}

impl external::External for Local {
    fn play_song(&mut self, mut song: song::Song) -> bool {
        if let external::ExternalType::LOCAL(ref mut local_song) = song.song_type {
            let file = load_file(&local_song.curr_song_path);
            let sink = play_file(&self.output_stream, file);
            local_song.playing = Playing::Playing(sink);

            self.current_song = Some(song);
            true
        } else {
            false // Not a local song type
        }
    }

    fn pause(&self) -> bool {
        // Implementation here
        true
    }

    fn play(&self) -> bool {
        // Implementation here
        true
    }

    fn stop(&self) -> Option<song::Song> {
        // Implementation here
        None
    }

    fn new() -> Self {
        Local {
            current_song: None,
            output_stream: rodio::OutputStreamBuilder::open_default_stream()
            .expect("open default audio stream"),
        }
    }
}

fn load_file(file_path: &str) -> BufReader<File> {
    BufReader::new(File::open(file_path).unwrap())
}

fn play_file(stream_handle: &OutputStream, file: BufReader<File>) -> Sink {
    rodio::play(&stream_handle.mixer(), file).unwrap()
}