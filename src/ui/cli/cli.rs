use std::sync::mpsc::Sender;
use std::u64;
use anyhow::anyhow;

use crate::failure::failure::{Failure, FailureType};
use crate::ui::cli::cli_ui::run_ui;
use crate::ui::ui;
use crate::Command;
use crate::{external::external::ExternalType, internal::{playlist::Playlist, song::Song}};
    
fn run_command(input: String, transmit: &Sender<Command>) -> Result<bool, Failure> {
    match input.trim().split_once(" ") {
        Some((command, args)) => {command_check_composite(command, args, transmit)}
        None => {command_check_single(input.trim(), transmit)}
    }
}

pub fn run_cli(transmit: Sender<Command>, update_interval: u64) {

    // input thread
    let (input_tx, input_rx) = std::sync::mpsc::channel::<String>();
    

    let renderer = run_ui(&transmit, input_tx, update_interval);


    println!("Please enter command or type 'help' for help.");

    for input in input_rx {
        if input.is_empty() {continue;}
        match run_command(input, &transmit) {
            Ok(quit) => if quit { break; },
            Err(err) => print_failure(err),
        }
    }

    let _ = renderer.join();
}

fn command_check_single(command: &str, transmit: &Sender<Command>) -> Result<bool, Failure> {
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
        "queue" => {
                    let songs = ui::queue_get_songs(transmit)?;
                    for (index, song) in songs.iter().enumerate() {
                        println!("{}: {} - {}", index, song.info.artist, song.info.title);
                    }}
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
        _ => {Err(Failure::from((anyhow!("Unknown command: {}", command), FailureType::Warning)))?;}
    }
    Ok(false)
}

fn command_check_composite(command: &str, args: &str, transmit: &Sender<Command>) -> Result<bool, Failure> {
    match command {
        "play" => {
            match args.split_once(" ") {
                Some((action, args)) => {
                    match action {
                        "new" => {
                            let song = Song::new(args)?;
                            ui::play_new(transmit, song)?;
                        }
                        _ => {return Err(Failure::from((anyhow!("Unknown command: play {} {}", action, args), FailureType::Warning)))?;}
                    }
                }
                None => match args {
                    "help" => {
                        println!("avaliliable play commands:
                        play: play the current song
                        play new <song>: play a new song from the given path
                        play help: display this help message for play commands");
                    }
                    _ => {return Err(Failure::from((anyhow!("Unknown command: play {}", args), FailureType::Warning)))?;}
                }
            }
        }
        "queue" => {
            handle_queue(transmit, args)?;
        }
        "playlist" => {
            handle_playlist(transmit, args)?;
        }
        _ => {return Err(Failure::from((anyhow!("Unknown command: {}", command), FailureType::Warning)))?;}
    }
    Ok(false)
}

fn handle_queue(transmit: &Sender<Command>, args: &str) -> Result<bool, Failure> {
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
                        Err(e) => {return Err(Failure::from((e.into(), FailureType::Warning)))?;}
                    })?;
                }
                "playlist" => {
                    let playlist = Playlist::load(args)?;
                    ui::queue_playlist(transmit, playlist)?;
                }
                _ => {return Err(Failure::from((anyhow!("Unknown command: queue {} {}", action, args), FailureType::Warning)))?;}
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
            _ => {return Err(Failure::from((anyhow!("Unknown command: queue {}", args), FailureType::Warning)))?;}
        }
    }
    Ok(false)
}

fn handle_playlist(transmit: &Sender<Command>, args: &str) -> Result<bool, Failure> {
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
                        Err(e) => {return Err(Failure::from((anyhow!("Invalid song index: {}", e), FailureType::Warning)));}
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
                                _ => {return Err(Failure::from((anyhow!("Invalid song indices: from: {}, to: {}", from_str, to_str), FailureType::Warning)));} 
                            }
                        }
                        None => {return Err(Failure::from((anyhow!("Invalid input for move command: {}", args), FailureType::Warning)))?;}
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
                    let index: usize = args.parse().map_err(|e: std::num::ParseIntError| Failure::from((e.into(), FailureType::Warning)))?;
                    let song = ui::playlist_get_song(transmit, index)?;
                    println!("{} - {}", song.info.artist, song.info.title);
                }
                "name" => {
                    ui::playlist_set_name(transmit, args)?;
                }
                _ => {return Err(Failure::from((anyhow!("Unknown command: playlist {} {}", action, args), FailureType::Warning)));}
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
                _ => return Err(Failure::from((anyhow!(format!("Unknown command: playlist {}", args)), FailureType::Warning))),
            }
        }
    }
    Ok(false)
}
                
pub fn print_failure(err: Failure) {
    println!("{}\n use help for help", err);
}