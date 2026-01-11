use std::fs::File;
use std::io::BufReader;
use rodio::{Decoder, OutputStream, Sink, OutputStreamBuilder};

use crate::external::{external::{self, External, ExternalSong::LOCAL}};
use crate::internal::song::Song;

pub struct Local{
    output_stream: OutputStream,
    sink: Sink,
}

#[derive(Clone, Debug)]
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
    fn play_new(&self, song: &Song) -> bool {
        if self.load_new(song) {
            self.sink.play();
            true
        } else {false}
    }

    fn pause(&self) -> bool {
        self.sink.pause();
        true
    }

    fn play(&self) -> bool {
        self.sink.play();
        true
    }

    fn stop(&self) -> bool {
        self.sink.stop();
        true
    }

}

impl Local {
    fn load_new(&self, song: &Song) -> bool {
        if let LOCAL(ref local_song) = song.song_type {
            self.stop();
            let decoder = load_decoder(&local_song.song_path);
            self.sink.append(decoder);
            true
        } else {false}
    }

    pub fn new(song:&Song) -> Local {
        let output_stream = OutputStreamBuilder::open_default_stream()
            .expect("open default audio stream");
        let sink = Sink::connect_new(&output_stream.mixer());
        sink.pause();
        let local = Local {
            output_stream,
            sink,
        };
        local.load_new(song);
        local
    }
}




fn load_decoder(file_path: &str) -> Decoder<BufReader<File>> {
    Decoder::new(BufReader::new(File::open(file_path).unwrap())).unwrap()
}