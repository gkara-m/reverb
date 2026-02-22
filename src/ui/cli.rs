use std::io::{self, BufRead};
use std::sync::mpsc::Sender;

use crate::{Command, ui::ui};
use crate::{external::external::ExternalType, internal::{playlist::Playlist, song::Song}};


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
    
fn run_command(input: String, transmit: &Sender<Command>) -> Result<bool, String> {
    match input.trim().split_once(" ") {
        Some((command, args)) => {command_check_composite(command, args, transmit)}
        None => {command_check_single(input.trim(), transmit)}
    }
}

pub fn run_cli(transmit: Sender<Command>) {
    println!("Please enter command or type 'help' for help.");

    loop {
        let input = get_input();
        if input.is_empty() {continue;}
        match run_command(input, &transmit) {
            Ok(quit) => if quit { break; },
            Err(err) => invalid_input(err),
        }
    }
}

fn command_check_single(command: &str, transmit: &Sender<Command>) -> Result<bool, String> {
    match command {
        "play" => {ui::play(transmit)?;}
        "pause" => {ui::pause(transmit)?;}
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
        "quit" | "q" | ":q" => {
            ui::shutdown(transmit)?;
            return Ok(true);
        }
        "queue" => {ui::queue_list(transmit)?;}
        "playlist" => {
                    println!("{}:", ui::playlist_get_name(transmit)?);
                    let songs = ui::playlist_get_songs(transmit)?;
                    for (index, song) in songs.iter().enumerate() {
                        println!("{}: {} - {}", index, song.info.artist, song.info.title);
                    }
                }
        "skip" => {ui::queue_next(transmit)?;}
        "song" => {
            let song = ui::current_song(transmit)?;
            println!("Currently playing: {} - {}", song.info.artist, song.info.title);
        }
        _ => {Err(format!("Unknown command: {}", command))?;}
    }
    Ok(false)
}

fn command_check_composite(command: &str, args: &str, transmit: &Sender<Command>) -> Result<bool, String> {
    match command {
        "play" => {
            match args.split_once(" ") {
                Some((action, args)) => {
                    match action {
                        "new" => {
                            let song = Song::new(args)?;
                            ui::play_new(transmit, song)?;
                        }
                        _ => {return Err(format!("Unknown command: play {} {}", action, args));}
                    }
                }
                None => match args {
                    "help" => {
                        println!("avaliliable play commands:
                        play: play the current song
                        play new <song>: play a new song from the given path
                        play help: display this help message for play commands");
                    }
                    _ => {return Err(format!("Unknown command: play {}", args))}
                }
            }
        }
        "queue" => {
            handle_queue(transmit, args)?;
        }
        "playlist" => {
            handle_playlist(transmit, args)?;
        }
        _ => {return Err(format!("Unknown command: {}", command));}
    }
    Ok(false)
}

fn handle_queue(transmit: &Sender<Command>, args: &str) -> Result<bool, String> {
    match args.split_once(" ") {
        Some((action, args)) => {
            match action {
                "add" => {
                    let song = Song::new(args)?;
                    ui::queue_add(transmit, song)?;
                }
                "remove" => {
                    ui::queue_remove(transmit, match args.parse() {
                        Ok(index) => index,
                        Err(e) => {return Err(format!("Invalid song index: {}", e));}
                    })?;
                }
                "playlist" => {
                    let playlist = Playlist::load(args)?;
                    ui::queue_playlist(transmit, playlist)?;
                }
                _ => {return Err(format!("Unknown command: queue {} {}", action, args));}
            }
        }
        None => match args {
            "help" => {
                println!("avaliliable queue commands:
                queue: list the current song queue
                queue add <song_type> <song>: add a song to the queue
                queue remove <index>: remove a song from the queue at the given index
                queue playlist <playlist_name>: add all songs from the given playlist to the queue
                queue playlist: add all songs from the current playlist to the queue
                queue help: display this help message for queue commands");
            }
            "playlist" => {
                ui::queue_current_playlist(transmit)?;
            }
            _ => {return Err(format!("Unknown command: queue {}", args));}
        }
    }
    Ok(false)
}

fn handle_playlist(transmit: &Sender<Command>, args: &str) -> Result<bool, String> {
    match args.split_once(" ") {
        Some((action, args)) => {
            match action {
                "add" => {
                    let song = Song::new(args)?;
                    ui::playlist_add(transmit, song)?;
                }
                "remove" => {
                    match args.parse() {
                        Ok(index) => {ui::playlist_remove(transmit, index)?;}
                        Err(e) => {return Err(format!("Invalid song index: {}", e));}
                    }
                }
                "load" => {
                    ui::playlist_load(transmit, args)?;
                }
                "move" => {
                    match args.split_once(" ") {
                        Some((from_str, to_str)) => {
                            match (from_str.parse(), to_str.parse()) {
                                (Ok(from), Ok(to)) => {ui::playlist_move_song(transmit, from, to)?;}
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
                            ui::playlist_new(transmit, name, Some(external_type))?;
                        }
                        None => {ui::playlist_new(transmit, args, None)?;}
                    }
                }
                "get" => {
                    let index: usize = args.parse().map_err(|_| format!("Invalid song index: {}", args))?;
                    let song = ui::playlist_get_song(transmit, index)?;
                    println!("{} - {}", song.info.artist, song.info.title);
                }
                "name" => {
                    ui::playlist_set_name(transmit, args)?;
                }
                _ => {return Err(format!("Unknown command: playlist {} {}", action, args));}
            }
        }
        None => {
            match args {
                "name" => {
                    println!("{}", ui::playlist_get_name(transmit)?);
                }
                "help" => {
                    println!("avaliliable playlist commands:
                    playlist: list all songs in the current playlist
                    playlist new <name> [external_type]: create a new playlist with the given name and optional external type
                    playlist load <name>: load the playlist with the given name
                    playlist add <song_type> <song>: add a song to the current playlist
                    playlist remove <index>: remove a song from the current playlist at the given index
                    playlist move <from_index> <to_index>: move a song in the current playlist from one index to another
                    playlist get <index>: list the song at the given index in the current playlist
                    playlist name: prints the current playlist name
                    playlist name <new_name>: set the name of the current playlist
                    playlist help: display this help message for playlist commands");
                }
                _ => return Err(format!("Unknown command: playlist {}", args)),
            }
        }
    }
    Ok(false)
}
                
pub fn invalid_input(err_msg: String) {
    println!("{}\n use help for help", err_msg);
}