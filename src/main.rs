// use std::fs::File;
// use std::io::BufReader;
// use rodio::{Decoder, OutputStream, source::Source, Sink};
// use std::mem;
use ui::cli;


mod external;
mod ui;
// use external::*;

fn main () {
    // Get an output stream handle to the default physical sound device.
    // Note that the playback stops when the stream_handle is dropped.
    let stream_handle = rodio::OutputStreamBuilder::open_default_stream()
            .expect("open default audio stream");

    // Load a sound from a file, using a path relative to Cargo.toml
    //let file = BufReader::new(File::open("sample/sf.flac").unwrap());
    let file_path = "sample/sf.flac";
    let file = external::local::load_file(file_path);

    // Note that the playback stops when the sink is dropped
    // let sink = rodio::play(&stream_handle.mixer(), file).unwrap();
    let mut song = external::local::play_file(&stream_handle, file);

    // The sound plays in a separate audio thread,
    // so we need to keep the main thread alive while it's playing.
    // std::thread::sleep(std::time::Duration::from_secs(10));

    cli::run_cli(&mut song, &stream_handle);

}