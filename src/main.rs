use ui::cli;

mod external;
mod ui;

use external::*;
use crate::external::{external::External, local};

mod internal;

fn main () {
    let file_path = "sample/sf.mp3";
    let mut local = external::local::Local::new();

    let song_struct = internal::song::Song {
        song_type: external::external::ExternalType::LOCAL(
            local::LocalSong::new(file_path)
        ),
        title: String::from("SF"),
        artist: String::from("artist??"),
    };

    // Note that the playback stops when the sink is dropped
    // let sink = rodio::play(&stream_handle.mixer(), file).unwrap();
    // let mut song = external::local::play_file(&stream_handle, file);
    local.play_song(song_struct);

    // The sound plays in a separate audio thread,
    // so we need to keep the main thread alive while it's playing.
    // std::thread::sleep(std::time::Duration::from_secs(10));

    cli::run_cli(&mut song, &stream_handle);

}