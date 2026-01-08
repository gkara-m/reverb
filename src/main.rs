use ui::cli;
use external::{external::{External, ExternalType::LOCAL}, local};
use internal::song::Song;

mod external;
mod ui;
mod internal;

fn main () {
    let file_path = "sample/sf.mp3";
    let mut local = local::Local::new();

    let song_struct = Song {
        song_type: LOCAL(
            local::LocalSong::new(file_path)
        ),
        title: String::from("SF"),
        artist: String::from("artist??"),
    };

    // let sink = rodio::play(&stream_handle.mixer(), file).unwrap();
    // let mut song = external::local::play_file(&stream_handle, file);

    // The sound plays in a separate audio thread,
    // so we need to keep the main thread alive while it's playing.
    // std::thread::sleep(std::time::Duration::from_secs(10));

    local.play_song(song_struct);

    cli::run_cli(&mut local);

}