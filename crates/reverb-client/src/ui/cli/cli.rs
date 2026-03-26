use anyhow::anyhow;
use std::sync::mpsc::Sender;
use std::sync::{Mutex, Arc};
use once_cell::sync::Lazy;

use crate::ui::cli::startup::{self, Startup};
use crate::{Command, DATA_FOLDER};
use crate::ui::cli::cli_ui::run_ui;
use crate::ui::cli::command_spec::CommandCallType::{Args, NoArgs, NotCallable};
use crate::ui::cli::command_spec::CommandSpec;
use crate::ui::ui;
use crate::{
    external::external::ExternalType,
    internal::{playlist::Playlist, song::Song},
};
use crate::MAIN_SENDER;

use reverb_core::failure::failure::{Failure, FailureType};

pub(super) static PLAYLIST: Lazy<Arc<Mutex<String>>> = Lazy::new(|| Arc::new(Mutex::new("default playlist".to_string())));

pub fn run_cli(update_interval: u64) -> Result<(), Failure> {

    // load the last played playlist
    let startup = Startup::load()?;
    *PLAYLIST.lock().unwrap() = startup.last_played_playlist;
    
    let command_spec = CommandSpec::new()
    // top-level commands
    .add("play", vec!["play"], " : Play the current song", Some(|_| ui::play()), NoArgs, None)
    .add("pause", vec!["pause"], " : Pause the current song", Some(|_| ui::pause()), NoArgs, None)
    .add("play and pause", vec!["playpause", "p"], " : Play or pause the current song", Some(|_| {
        if ui::is_song_playing()? {
            ui::pause()
        } else {
            ui::play()
        }
    }), NoArgs, None)
    .add("quit", vec!["quit", "exit", ":q", "x"], " : Quit the application", Some(|_| ui::shutdown()), NoArgs, None)
    .add("help", vec!["help", "h"], " : Display this help message", Some(|_| Ok(())), NoArgs, None)
    .add("skip", vec!["skip", "s"], " : Skip the current song", Some(|_| ui::queue_next()), NoArgs, None)
    .add("song", vec!["song"], " : Display the currently playing song", Some(|_| {
        let song = ui::current_song()?;
        println!("Currently playing: {} - {}", song.info.artists[0], song.info.title);
        Ok(())
    }), NoArgs, None)
    // queue commands
    .add("queue", vec!["queue", "q"], " : List the current song queue", Some(|_| {
        let songs = ui::queue_get_songs()?;
        for (index, song) in songs.iter().enumerate() {
            println!("{}: {} - {}", index, song.info.artists[0], song.info.title);
        }
        Ok(())
    }), NoArgs, None)
    .add("queue add", vec!["add", "a"], " <song_type> <song_info> : Add a song to the queue", Some(|args| {
        let song = Song::new(args)?;
        ui::queue_add(song)
    }), Args, Some("queue"))
    .add("queue remove", vec!["remove", "r"], " <index> : Remove a song from the queue", Some(|args| {
        let index: usize = args.parse().map_err(|e: std::num::ParseIntError| Failure::from((e.into(), FailureType::Warning)))?;
        ui::queue_remove(index)
    }), Args, Some("queue"))
    .add("queue clear", vec!["clear", "c", "cl"], " : Clear the queue", Some(|_| ui::queue_clear()), NoArgs, Some("queue"))
    .add("queue load playlist", vec!["load"], " <playlist_name> : Load a playlist into the queue", Some(|args| {
        ui::queue_playlist(args)
    }), Args, Some("queue"))
    .add("queue current playlist", vec!["playlist", "p"], " : Add current playlist songs to the queue", Some(|_| ui::queue_playlist(PLAYLIST.lock().unwrap().as_str())), NoArgs, Some("queue"))
    .add("queue shuffle", vec!["shuffle", "sh", "s"], " : Shuffle the queue", Some(|_| ui::queue_shuffle()), NoArgs, Some("queue"))
    // play sub-commands
    .add("play new", vec!["new", "n"], " <song-type> <song-info> : Play a new song from the given info", Some(|args| {
        let song = Song::new(args)?;
        ui::play_new(song)
    }), Args, Some("play"))
    // playlist commands
    .add("playlist", vec!["playlist", "p", "pl"], " : List the current playlist", Some(|_| {
        println!("{}:", PLAYLIST.lock().unwrap().as_str());
        let songs = ui::playlist_get_songs(PLAYLIST.lock().unwrap().as_str())?;
        for (index, song) in songs.iter().enumerate() {
            println!("{}: {} - {}", index, song.info.artists[0], song.info.title);
        }
        Ok(())
    }), NoArgs, None)
    .add("playlist add", vec!["add", "a"], " <song_type> <song_info> : Add a song to the playlist (or 'playlist add playlist <name>')", Some(|args| {
        if args.starts_with("playlist ") {
            let name = args.strip_prefix("playlist ").unwrap();
            ui::playlist_add_playlist(PLAYLIST.lock().unwrap().as_str(), name)
        } else {
            let song = Song::new(args)?;
            ui::playlist_add(PLAYLIST.lock().unwrap().as_str(), song)
        }
    }), Args, Some("playlist"))
    .add("playlist remove", vec!["remove", "r"], " <index> : Remove a song from the playlist", Some(|args| {
        let index: usize = args.parse().map_err(|e: std::num::ParseIntError| Failure::from((e.into(), FailureType::Warning)))?;
        ui::playlist_remove(PLAYLIST.lock().unwrap().as_str(), index - 1)
    }), Args, Some("playlist"))
    .add("playlist load", vec!["load", "l"], " <name> : Load a playlist by name", Some(|args| {*PLAYLIST.lock().unwrap() = args.to_string(); Ok(())}), Args, Some("playlist"))
    .add("playlist move", vec!["move", "m"], " <from> <to> : Move a song in the playlist", Some(|args| {
        match args.split_once(' ') {
            Some((from_str, to_str)) => {
                match (from_str.parse::<usize>(), to_str.parse::<usize>()) {
                    (Ok(from), Ok(to)) => ui::playlist_move_song(PLAYLIST.lock().unwrap().as_str(), from - 1, to - 1),
                    _ => Err(Failure::from((anyhow!("Invalid song indices: from: {}, to: {}", from_str, to_str), FailureType::Warning))),
                }
            }
            None => Err(Failure::from((anyhow!("Invalid input for move command: {}", args), FailureType::Warning))),
        }
    }), Args, Some("playlist"))
    .add("playlist new", vec!["new", "n"], " <name> [external_type] : Create a new playlist", Some(|args| {
        match args.split_once(' ') {
            Some((name, external_type)) => {
                let external_type = ExternalType::get_from_str(external_type)?;
                ui::playlist_new(name, Some(external_type))
            }
            None => ui::playlist_new(args, None),
        }
    }), Args, Some("playlist"))
    .add("playlist get", vec!["get", "g"], " <index> : Get a song by index", Some(|args| {
        let index: usize = args.parse().map_err(|e: std::num::ParseIntError| Failure::from((e.into(), FailureType::Warning)))?;
        let song = ui::playlist_get_song(PLAYLIST.lock().unwrap().as_str(), index - 1)?;
        println!("{} - {}", song.info.artists[0], song.info.title);
        Ok(())
    }), Args, Some("playlist"))
    .add("playlist name", vec!["name"], " : Print the current playlist name", Some(|_| {
        println!("{}", PLAYLIST.lock().unwrap().as_str());
        Ok(())
    }), NoArgs, Some("playlist"))
    .add("playlist rename", vec!["rename", "set-name", "rn"], " <new_name> : Set the playlist name", Some(|args| {ui::playlist_set_name(PLAYLIST.lock().unwrap().as_str(), args)?; *PLAYLIST.lock().unwrap() = args.to_string(); Ok(())}), Args, Some("playlist"))
    .add("playlist copy", vec!["copy", "c"], " <new_name> : Copy the playlist", Some(|args| ui::playlist_copy_to(PLAYLIST.lock().unwrap().as_str(), args)), Args, Some("playlist"))
    .add("playlist clear", vec!["clear", "cl"], " : Clear the playlist", Some(|_| ui::playlist_clear(PLAYLIST.lock().unwrap().as_str())), NoArgs, Some("playlist"))
    // server commands
    .add("server", vec!["server"], " : Server related commands", None, NotCallable, None)
    .add("server add", vec!["add"], " <name> <address> <certificate_path> : Add a server configuration", Some(|args| {
        let mut parts = args.splitn(3, ' ');
        match (parts.next(), parts.next(), parts.next()) {
            (Some(name), Some(address), Some(certificate_path)) => ui::add_server(name.to_string(), address.to_string(), certificate_path.to_string()),
            _ => Err(Failure::from((anyhow!("Invalid input for server add command: {}", args), FailureType::Warning))),
        }
    }), Args, Some("server"))
    .add("server connect", vec!["connect", "con"], " : Connect to the server", Some(|_| ui::connect_to_server()), NoArgs, Some("server"))
    .add("server query", vec!["query", "qry"], " <message> : Send a query to the server", Some(|args| ui::send_query(args)), Args, Some("server"))
    .add("server notify", vec!["notify", "send", "snd"], " <message> : Notify the server", Some(|args| ui::send_notify(args)), Args, Some("server"));


    // input thread
    let (input_tx, input_rx) = std::sync::mpsc::channel::<String>();

    let renderer = run_ui(&MAIN_SENDER.get().unwrap(), input_tx, update_interval);


    println!("Please enter command or type 'help' for help.");

    for input in input_rx {
        match command_spec.call(input.as_str()) {
            Err(err) => print_failure(err),
            Ok(_) => {}
        }
    }

    let _ = renderer.join();
    Ok(())
}

pub fn print_failure(err: Failure) {
    println!("{}\n use help for help", err);
}
