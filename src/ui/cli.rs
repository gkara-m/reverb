use std::io::{self, BufRead};
use crate::{external::{external::{External, ExternalSong::LOCAL}, local::{self, Local}}, internal::{internal::Internal, song::Song}};

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

    
    fn run_command(input: String, internal: &mut Internal) {
        match input.trim() {
        "play" => {internal.play();}
        "pause" => {internal.pause();}
        "play-new" => {
            internal.play_new(Song {
                song_type: LOCAL(
                    local::LocalSong::new("sample/sf.mp3")
                ),
                title: String::from("SF"),
                artist: String::from("artist??"),
            });
        }
        // "stop" => {internal.stop();}
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

pub fn run_cli(internal: &mut Internal) {
    println!("Please enter command or type 'help' for help.");

    loop {
        let input = get_input();
        if input.is_empty() {continue;}
        run_command(input, internal);
    }
}