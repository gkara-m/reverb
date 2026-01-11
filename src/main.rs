use ui::cli;
use internal::{internal::Internal, song::Song};
use external::{external::ExternalSong, local::LocalSong};

mod external;
mod ui;
mod internal;

fn main () {

    let default_song = Song {
        song_type: ExternalSong::LOCAL(
            LocalSong::new("sample/sf.mp3")
            .expect("Failed to find default sample/sf.mp3 file")
        ),
        title: String::from("SF"),
        artist: String::from("artist??"),
    };


    let mut internal = Internal::new(default_song);

    cli::run_cli(&mut internal);

}