use ui::cli;
use internal::{internal::Internal, song::Song};
use external::{external::ExternalSong, local::LocalSong};

mod external;
mod ui;
mod internal;

pub static PLAYLIST_FOLDER: &str = "playlists/";

fn main () {

    

    let default_song = Song {
        song_type: ExternalSong::LOCAL(
            LocalSong::new("sample/sf.mp3")
            .expect("Failed to find default sample/sf.mp3 file")
        ),
        title: String::from("SF"),
        artist: String::from("artist??"),
    };


    let mut internal = match Internal::new(default_song) {
        Err(e) => {
            println!("Failed to initialize internal state: {}", e);
            return;
        }
        Ok(internal) => { internal }
    };

    cli::run_cli(&mut internal);

}