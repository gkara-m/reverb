use std::io::{self, BufRead};
use crate::{external::external::ExternalType, internal::{internal::Internal, song::Song}};


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
    
    fn run_command(input: String, internal: &mut Internal) -> Result<bool, String> {
        match input.trim().split_once(" ") {
        Some((command, args)) => {command_check_composite(command, args, internal)}
        None => {command_check_single(input.trim(), internal)}
    }
}

pub fn run_cli(internal: &mut Internal) {
    println!("Please enter command or type 'help' for help.");

    loop {
        let input = get_input();
        if input.is_empty() {continue;}
        match run_command(input, internal) {
            Ok(quit) => if quit { break; },
            Err(err) => invalid_input(err),
        }
    }
}

fn command_check_single(command: &str, internal: &mut Internal) -> Result<bool, String> {
    match command {
        "play" => {internal.play()?;}
        "pause" => {internal.pause()?;}
        "help" | "h" => {println!("No help for you! GET GOOD LOOSER");}
        "quit" | "q" | ":q" => {return Ok(true);}
        "queue" => {internal.queue_list()?;}
        "skip" => {internal.queue_next()?;}
        _ => {Err(format!("Unknown command: {}", command))?;}
    }
    Ok(false)
}

fn command_check_composite(command: &str, args: &str, internal: &mut Internal) -> Result<bool, String> {
    match command {
        "play" => {
            match args {
                "new" => {
                    let song = Song::new(args)?;
                    internal.play_new(song)?;
                }
                _ => {return Err(format!("Unknown play command: {}", args))}
            }
        }
        "queue" => {
            handle_queue(internal, args)?;
        }
        "playlist" => {
            handle_playlist(internal, args)?;
        }
        _ => {return Err(format!("Unknown command: {}", command));}
    }
    Ok(false)
}

fn handle_queue(internal: &mut Internal, args: &str) -> Result<bool, String> {
    match args {
        "add" => {
            let song = Song::new(args)?;
            internal.queue_add(song)?;
        }
        "remove" => {
            internal.queue_remove(match args.parse() {
                Ok(index) => index,
                Err(e) => {return Err(format!("Invalid song index: {}", e));}
            })?;
        }
        _ => {return Err(format!("Unknown queue command: {}", args));}
    }
    Ok(false)
}

fn handle_playlist(internal: &mut Internal, args: &str) -> Result<bool, String> {
    match args.split_once(" ") {
        Some((action, args)) => {
            match action {
                "add" => {
                    let song = Song::new(args)?;
                    internal.playlist_add(song)?;
                }
                "remove" => {
                    match args.parse() {
                        Ok(index) => {internal.playlist_remove(index)?;}
                        Err(e) => {return Err(format!("Invalid song index: {}", e));}
                    }
                }
                "load" => {
                    internal.load_playlist(args)?;
                }
                "move" => {
                    match args.split_once(" ") {
                        Some((from_str, to_str)) => {
                            match (from_str.parse(), to_str.parse()) {
                                (Ok(from), Ok(to)) => {internal.playlist_move_song(from, to)?;}
                                _ => {return Err(format!("Invalid song indices: from: {}, to: {}", from_str, to_str));} 
                            }
                        }
                        None => {return Err(format!("Invalid input for move command: {}", args));}
                    }
                }
                "list" => {
                    let songs = internal.playlist_get_songs()?;
                    for (index, song) in songs.iter().enumerate() {
                        println!("{}: {} - {}", index, song.artist, song.title);
                    }
                }
                "new" => {
                    match args.split_once(" ") {
                        Some((name, external_type)) => {
                            let external_type =  ExternalType::get_from_str(external_type)?;
                            internal.new_playlist(name, Some(external_type))?;
                        }
                        None => {internal.new_playlist(args, None)?;}
                    }
                }
                _ => {return Err(format!("Unknown playlist command: {}", args));}
            }
        }
        None => {return Err(format!("No playlist command provided"));}
    }
    Ok(false)
}
                
fn invalid_input(err_msg: String) {
    println!("{}\n use help for help", err_msg);
}