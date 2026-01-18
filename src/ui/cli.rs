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
        "help" | "h" => {println!("
        project is a WIP help may be out of date:
        avaliliable commands:
        play (composite): play commands
        pause: used to pause the current song
        help: display this help message
        quit: quit the application
        skip: skip the current song
        playlist (composite): manage playlists
        queue (composite): manage the song queue
        use \"<command> help\" for more detailed help for composite commands
        source code available at: https://github.com/SixOneFiveZero/reverb
        ");}
        "quit" | "q" | ":q" => {return Ok(true);}
        "queue" => {internal.queue_list()?;}
        "playlist" => {
                    let songs = internal.playlist_get_songs()?;
                    for (index, song) in songs.iter().enumerate() {
                        println!("{}: {} - {}", index, song.artist, song.title);
                    }
                }
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
                "help" => {
                    println!("avaliliable play commands:
                    play: play the current song
                    play new <song>: play a new song from the given path
                    play help: display this help message for play commands");
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
        "help" => {
            println!("avaliliable queue commands:
            queue: list the current song queue
            queue add <song_type> <song>: add a song to the queue
            queue remove <index>: remove a song from the queue at the given index
            queue help: display this help message for queue commands");
        }
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
                "new" => {
                    match args.split_once(" ") {
                        Some((name, external_type)) => {
                            let external_type =  ExternalType::get_from_str(external_type)?;
                            internal.new_playlist(name, Some(external_type))?;
                        }
                        None => {internal.new_playlist(args, None)?;}
                    }
                }
                "get" => {
                    let index: usize = args.parse().map_err(|_| format!("Invalid song index: {}", args))?;
                    let song = internal.playlist_get_song(index)?;
                    println!("{} - {}", song.artist, song.title);
                }
                _ => {return Err(format!("Unknown playlist command: {}", args));}
            }
        }
        None => {
            match args {
                "help" => {
                    println!("avaliliable playlist commands:
                    playlist: list all songs in the current playlist
                    playlist new <name> [external_type]: create a new playlist with the given name and optional external type
                    playlist load <name>: load the playlist with the given name
                    playlist add <song_type> <song>: add a song to the current playlist
                    playlist remove <index>: remove a song from the current playlist at the given index
                    playlist move <from_index> <to_index>: move a song in the current playlist from one index to another
                    playlist get <index>: list the song at the given index in the current playlist
                    playlist help: display this help message for playlist commands");
                }
                _ => return Err(format!("invalid playlist command provided")),
            }
        }
    }
    Ok(false)
}
                
fn invalid_input(err_msg: String) {
    println!("{}\n use help for help", err_msg);
}