use std::sync::mpsc::Sender;
use anyhow::anyhow;

use crate::failure::failure::{Failure, FailureType};
use crate::ui::cli::cli_ui::run_ui;
use crate::ui::cli::command_spec::CommandSpec;
use crate::ui::cli::command_spec::CommandCallType::{Args, NoArgs};
use crate::ui::ui;
use crate::Command;
use crate::{external::external::ExternalType, internal::{playlist::Playlist, song::Song}};

pub fn run_cli(transmit: Sender<Command>, update_interval: u64) {
    let command_spec = CommandSpec::new(transmit.clone())
    // top-level commands
    .add("play", vec!["play"], " : Play the current song", Some(|_, tx| ui::play(tx)), NoArgs, None)
    .add("pause", vec!["pause"], " : Pause the current song", Some(|_, tx| ui::pause(tx)), NoArgs, None)
    .add("quit", vec!["quit", "q", ":q"], " : Quit the application", Some(|_, tx| ui::shutdown(tx)), NoArgs, None)
    .add("help", vec!["help", "h"], " : Display this help message", Some(|_, _| Ok(())), NoArgs, None)
    .add("skip", vec!["skip"], " : Skip the current song", Some(|_, tx| ui::queue_next(tx)), NoArgs, None)
    .add("song", vec!["song"], " : Display the currently playing song", Some(|_, tx| {
        let song = ui::current_song(tx)?;
        println!("Currently playing: {} - {}", song.info.artist, song.info.title);
        Ok(())
    }), NoArgs, None)
    // queue commands
    .add("queue", vec!["queue"], " : List the current song queue", Some(|_, tx| {
        let songs = ui::queue_get_songs(tx)?;
        for (index, song) in songs.iter().enumerate() {
            println!("{}: {} - {}", index, song.info.artist, song.info.title);
        }
        Ok(())
    }), NoArgs, None)
    .add("queue add", vec!["add"], " <song_type> <song_info> : Add a song to the queue", Some(|args, tx| {
        let song = Song::new(args)?;
        ui::queue_add(tx, song)
    }), Args, Some("queue"))
    .add("queue remove", vec!["remove"], " <index> : Remove a song from the queue", Some(|args, tx| {
        let index: usize = args.parse().map_err(|e: std::num::ParseIntError| Failure::from((e.into(), FailureType::Warning)))?;
        ui::queue_remove(tx, index)
    }), Args, Some("queue"))
    .add("queue clear", vec!["clear"], " : Clear the queue", Some(|_, tx| ui::queue_clear(tx)), NoArgs, Some("queue"))
    .add("queue playlist", vec!["playlist", "load"], " <playlist_name> : Load a playlist into the queue", Some(|args, tx| {
        let playlist = Playlist::load(args)?;
        ui::queue_playlist(tx, playlist)
    }), Args, Some("queue"))
    .add("queue fill", vec!["fill"], " : Add current playlist songs to the queue", Some(|_, tx| ui::queue_current_playlist(tx)), NoArgs, Some("queue"))
    // play sub-commands
    .add("play new", vec!["new"], " <song-type> <song-info> : Play a new song from the given info", Some(|args, tx| {
        let song = Song::new(args)?;
        ui::play_new(tx, song)
    }), Args, Some("play"))
    // playlist commands
    .add("playlist", vec!["playlist"], " : List the current playlist", Some(|_, tx| {
        println!("{}:", ui::playlist_get_name(tx)?);
        let songs = ui::playlist_get_songs(tx)?;
        for (index, song) in songs.iter().enumerate() {
            println!("{}: {} - {}", index, song.info.artist, song.info.title);
        }
        Ok(())
    }), NoArgs, None)
    .add("playlist add", vec!["add"], " <song_type> <song_info> : Add a song to the playlist (or 'playlist add playlist <name>')", Some(|args, tx| {
        if args.starts_with("playlist ") {
            let name = args.strip_prefix("playlist ").unwrap();
            ui::playlist_add_playlist(tx, name)
        } else {
            let song = Song::new(args)?;
            ui::playlist_add(tx, song)
        }
    }), Args, Some("playlist"))
    .add("playlist remove", vec!["remove"], " <index> : Remove a song from the playlist", Some(|args, tx| {
        let index: usize = args.parse().map_err(|e: std::num::ParseIntError| Failure::from((e.into(), FailureType::Warning)))?;
        ui::playlist_remove(tx, index - 1)
    }), Args, Some("playlist"))
    .add("playlist load", vec!["load"], " <name> : Load a playlist by name", Some(|args, tx| ui::playlist_load(tx, args)), Args, Some("playlist"))
    .add("playlist move", vec!["move"], " <from> <to> : Move a song in the playlist", Some(|args, tx| {
        match args.split_once(' ') {
            Some((from_str, to_str)) => {
                match (from_str.parse::<usize>(), to_str.parse::<usize>()) {
                    (Ok(from), Ok(to)) => ui::playlist_move_song(tx, from - 1, to - 1),
                    _ => Err(Failure::from((anyhow!("Invalid song indices: from: {}, to: {}", from_str, to_str), FailureType::Warning))),
                }
            }
            None => Err(Failure::from((anyhow!("Invalid input for move command: {}", args), FailureType::Warning))),
        }
    }), Args, Some("playlist"))
    .add("playlist new", vec!["new"], " <name> [external_type] : Create a new playlist", Some(|args, tx| {
        match args.split_once(' ') {
            Some((name, external_type)) => {
                let external_type = ExternalType::get_from_str(external_type)?;
                ui::playlist_new(tx, name, Some(external_type))
            }
            None => ui::playlist_new(tx, args, None),
        }
    }), Args, Some("playlist"))
    .add("playlist get", vec!["get"], " <index> : Get a song by index", Some(|args, tx| {
        let index: usize = args.parse().map_err(|e: std::num::ParseIntError| Failure::from((e.into(), FailureType::Warning)))?;
        let song = ui::playlist_get_song(tx, index)?;
        println!("{} - {}", song.info.artist, song.info.title);
        Ok(())
    }), Args, Some("playlist"))
    .add("playlist name", vec!["name"], " : Print the current playlist name", Some(|_, tx| {
        println!("{}", ui::playlist_get_name(tx)?);
        Ok(())
    }), NoArgs, Some("playlist"))
    .add("playlist rename", vec!["rename", "set-name"], " <new_name> : Set the playlist name", Some(|args, tx| ui::playlist_set_name(tx, args)), Args, Some("playlist"))
    .add("playlist copy", vec!["copy"], " <new_name> : Copy the playlist", Some(|args, tx| ui::playlist_copy_to(tx, args)), Args, Some("playlist"))
    .add("playlist clear", vec!["clear"], " : Clear the playlist", Some(|_, tx| ui::playlist_clear(tx)), NoArgs, Some("playlist"));


    // input thread
    let (input_tx, input_rx) = std::sync::mpsc::channel::<String>();
    

    let renderer = run_ui(&transmit, input_tx, update_interval);


    println!("Please enter command or type 'help' for help.");

    for input in input_rx {
        match command_spec.call(input.as_str()) {
            Ok(quit) => if quit { break; },
            Err(err) => print_failure(err),
        }
    }

    let _ = renderer.join();
}

pub fn print_failure(err: Failure) {
    println!("{}\n use help for help", err);
}