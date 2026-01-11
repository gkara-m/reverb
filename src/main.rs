use ui::cli;
use internal::internal::Internal;

mod external;
mod ui;
mod internal;

fn main () {

    let default_song = internal::song::Song {
        song_type: external::external::ExternalSong::LOCAL(
            external::local::LocalSong::new("sample/sf.mp3")
            .expect("Failed to find default sample/sf.mp3 file")
        ),
        title: String::from("SF"),
        artist: String::from("artist??"),
    };


    let mut internal = Internal::new(default_song);

    cli::run_cli(&mut internal);

}