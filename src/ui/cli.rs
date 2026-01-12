use std::io::{self, BufRead};
use crate::{external::external::ExternalType, internal::{internal::Internal, playlist, song::Song}};

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
        Some((command, args)) => {command_check_composite(command, args, internal)}
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
        "play" => {internal.play();}
        "pause" => {internal.pause();}
        "help" | "h" => {println!("Placeholder help message");}
        "quit" | "q" | ":q" => {return true;}
        "queue" => {internal.queue_list();}
        "skip" => {internal.queue_next();}
        _ => {invalid_input();}
    }
    false
}

fn command_check_composite(command: &str, args: &str, internal: &mut Internal) -> bool {
    match command {
        "play-new" => {
            match Song::new(args) {
                Some(song) => {internal.play_new(song);},
                None => {invalid_input();}
            }
        }
        "queue-add" => {
            match Song::new(args) {
                Some(song) => {internal.queue_add(song);}
                None => {invalid_input();}
            }
        }
        "queue-remove" => {
            internal.queue_remove(args.parse().expect("Please enter valid song index"));
        }
        "playlist" => {
            handle_playlist(internal, args);
        }
        _ => {invalid_input();}
    }
    false
}

fn handle_playlist(internal: &mut Internal, args: &str) {
    match args.split_once(" ") {
        Some((action, args)) => {
            match action {
                "add" => {
                    match Song::new(args) {
                        Some(song) => {internal.playlist_add(song);}
                        None => {invalid_input();}
                    }
                }
                "remove" => {
                    match args.parse() {
                        Ok(index) => {internal.playlist_remove(index);}
                        Err(_) => {invalid_input();}
                    }
                }
                "load" => {
                    if !internal.load_playlist(args) {
                        print!("ahhhhh");
                    }
                }
                "move" => {
                    match args.split_once(" ") {
                        Some((from_str, to_str)) => {
                            match (from_str.parse(), to_str.parse()) {
                                (Ok(from), Ok(to)) => {internal.playlist_move_song(from, to);}
                                _ => {invalid_input();}
                            }
                        }
                        None => {invalid_input();}
                    }
                }
                "list" => {
                    let songs = internal.playlist_get_songs();
                    for (index, song) in songs.iter().enumerate() {
                        println!("{}: {} - {}", index, song.artist, song.title);
                    }
                }
                "new" => {
                    match args.split_once(" ") {
                        Some((name, external_type)) => {
                            match ExternalType::get_from_str(external_type) {
                                Some(external_type) => {
                                    if !internal.new_playlist(name, Some(external_type)) {
                                        print!("ahhhhh");
                                    }
                                }
                                None => {invalid_input();}
                            }
                        }
                        None => {internal.new_playlist(args, None);}
                    }
                }
                _ => {invalid_input();}
            }
        }
        None => {invalid_input();}
    }
}
                

fn invalid_input() {
    println!("Invalid Input");
}