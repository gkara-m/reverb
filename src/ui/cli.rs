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
        match input.trim().split_once(" ") {
        Some((command, param)) => {command_check_composite(command, param, internal)}
        None => {command_check_single(input.trim(), internal)}
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

fn command_check_single(command: &str, internal: &mut Internal) -> bool {
    match command {
        "play" => {internal.play(); false}
        "pause" => {internal.pause(); false}
        "help" | "h" => {println!("Placeholder help message"); false}
        "quit" | "q" | ":q" => {true}
        "queue" => {internal.queue_list(); false}
        "skip" => {internal.queue_next(); false}
        _ => return invalid_input()
    }
}

fn command_check_composite(command: &str, param: &str, internal: &mut Internal) -> bool {
    match command {
        "play-new" => {
            match Song::new(param) {
                Some(song) => {internal.play_new(song); return false},
                None => return invalid_input()
            }
        }
        "queue-add" => {
            match Song::new(param) {
                Some(song) => {internal.queue_add(song);}
                None => {invalid_input();}
                _ => {invalid_input();}
            }
            false
        }
        "queue-remove" => {
            internal.queue_remove(param.parse().expect("Please enter valid song index"));
            return false
        }
        _ => return invalid_input()
    }
}

fn invalid_input() -> bool {
    println!("Invalid Input");
    false
}