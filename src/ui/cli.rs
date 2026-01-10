use std::io::{self, BufRead};
use crate::{external::{external::{ExternalSong::LOCAL}, local}, internal::{internal::Internal, song::Song}};

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
    
    fn run_command(input: String, internal: &mut Internal) -> bool {
        match input.trim() {
        "play" => {internal.play(); false}
        "pause" => {internal.pause(); false}
        "play-new" => {
            internal.play_new(Song {
                song_type: LOCAL(
                    local::LocalSong::new("sample/sf.mp3")
                ),
                title: String::from("SF"),
                artist: String::from("artist??"),
            });
            false
        }
        "help" | "h" => {println!("Placeholder help message"); false}
        "quit" | "q" | ":q" => {true}
        "queue-add" => {println!("Not Implemented"); false}
        "queue-remove" => {println!("Not Implemented"); false}
        "queue" | "queue-list" => {internal.queue_list(); false}
        "queue-next" | "skip" => {internal.queue_next(); false}
        _ => {println!("Invalid input: {input}");false}
    }
}

pub fn run_cli(internal: &mut Internal) {
    println!("Please enter command or type 'help' for help.");

    'inputs: loop {
        let input = get_input();
        if input.is_empty() {continue;}
        let quit = run_command(input, internal);
        if quit {break 'inputs;};
    }
}