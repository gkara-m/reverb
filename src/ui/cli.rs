use std::io::{self, BufRead};
use crate::external::*;

// use rodio::OutputStream;
use rodio::{Decoder, OutputStream, source::Source, Sink};

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

fn run_command(input: String, song: &mut Sink, stream_handle: &OutputStream) {
    match input.trim() {
        "play" => {song.play()}
        "pause" => {song.pause()}
        "play-new" => {println!("Not implemented yet")}
        "stop" => {song.stop()}
        "help" | "h" => {println!("Placeholder help message")}
        "quit" | "q" | ":q" => {println!("Not implemented yet")}
        _ => {
            println!("Invalid input: {}", input);
            if let Some(("play-new", file_path)) = input.trim().split_once(" ") {
                let file = local::load_file(file_path);
                let mut song = local::play_file(stream_handle, file);
            }
        }
    }
}

pub fn run_cli(song: &mut Sink, stream_handle: &OutputStream) {
    println!("Please enter command or type 'help' for help.");

    loop {
        let input = get_input();
        if input.is_empty() {continue;}
        run_command(input, song, stream_handle);
    }
}

pub enum Command {
    Play,
    Pause,
    PlayNew,
    Stop,
    Help,
}
