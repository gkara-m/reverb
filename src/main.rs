use crate::external::{external::External, local};

mod external;
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

    local.play_song(song_struct);


    std::thread::sleep(std::time::Duration::from_secs(1));

    local.pause();

    std::thread::sleep(std::time::Duration::from_secs(1));

    local.play();

    std::thread::sleep(std::time::Duration::from_secs(1));

    local.stop();

    std::thread::sleep(std::time::Duration::from_secs(3));
}