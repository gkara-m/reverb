use std::io::{self, BufRead};
use crate::{external::{external::{External, ExternalSongType::LOCAL}, local::{self, Local}}, internal::song::Song};

pub fn get_input() -> String {
    let stdin = io::stdin();
    let mut buffer = String::new();
    let mut handle = stdin.lock();

    match handle.read_line(&mut buffer) {
        Ok(_) => {
            buffer.trim().to_string()
        }
        Err(e) => {
            eprintln!("Invalid Input: {}", e);
            String::new()
        }
    }
}

    
    fn run_command(input: String, local: &mut Local) {
        match input.trim() {
        "play" => {local.play();}
        "pause" => {local.pause();}
        "play-new" => {
            local.play_new(&Song {
                song_type: LOCAL(
                    local::LocalSong::new("sample/sf.mp3")
                ),
                title: String::from("SF"),
                artist: String::from("artist??"),
            });
        }
        "stop" => {local.stop();}
        "help" | "h" => {println!("Placeholder help message")}
        "quit" | "q" | ":q" => {println!("Not implemented yet")}
        _ => {
            println!("Invalid input: {input}");
            // if let Some(("play-new", file_path)) = input.trim().split_once(" ") {
            //     let file = local::load_file(file_path);
            //     let mut song = local::play_file(stream_handle, file);
            
        }
    }
}

pub fn run_cli(local: &mut Local) {
    println!("Please enter command or type 'help' for help.");

    loop {
        let input = get_input();
        if input.is_empty() {continue;}
        run_command(input, local);
    }
}