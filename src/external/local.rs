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
    song_path : String,
    sink: Option<Sink>,
}

impl LocalSong {
    pub fn new(song_path: &str) -> Self {
        LocalSong {
            song_path: song_path.to_string(),
            sink: None,
        }
    }
}


impl external::External for Local {
    fn play_song(&mut self, mut song: song::Song) -> bool {
        if let external::ExternalType::LOCAL(ref mut local_song) = song.song_type {
            let file = load_file(&local_song.song_path);
            let sink = play_file(&self.output_stream, file);
            local_song.sink = Some(sink);

            self.current_song = Some(song);
            true
        } else {
            false // Not a local song type
        }
    }

    fn pause(&self) -> bool {
        if let Some(ref song) = self.current_song {
            if let external::ExternalType::LOCAL(ref local_song) = song.song_type {
                if let Some(ref sink) = local_song.sink {
                    sink.pause();
                    return true;
                }
            }
        }
        false
    }

    fn play(&self) -> bool {
        if let Some(ref song) = self.current_song {
            if let external::ExternalType::LOCAL(ref local_song) = song.song_type {
                if let Some(ref sink) = local_song.sink {
                    sink.play();
                    return true;
                }
            }
        }
        false
    }

    fn stop(&mut self) -> Option<song::Song> {
        if let Some(ref mut song) = self.current_song {
            if let external::ExternalType::LOCAL(ref mut local_song) = song.song_type {
                if let Some(sink) = local_song.sink.take() {
                    std::mem::drop(sink);
                    return self.current_song.take();
                }
            }
        }
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