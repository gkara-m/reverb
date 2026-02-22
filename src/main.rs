use std::path::Path;
use std::sync::mpsc;
use std::thread;

use external::{external::ExternalSong, local::LocalSong};
use internal::{
    internal::Internal,
    song::Song,
};
use once_cell::sync::OnceCell;
use serde::{Deserialize, Serialize};
use toml;
use ui::cli;

use crate::{
    config::startup_shutdown::{shutdown, startup}, external::external::{ExternalSongTrait, ExternalType}, internal::{playlist::Playlist, queue::Queue}
};

mod external;
mod internal;
mod ui;
mod config;

pub static CONFIG_FOLDER: &str = "configs/";

pub static DATA_FOLDER: OnceCell<String> = OnceCell::new();
pub static LOCAL_SONG_FOLDER_PATH: OnceCell<String> = OnceCell::new();

fn main() {
    let (transmit, receive) = mpsc::channel::<Command>();

    let mut internal = match startup(transmit.clone()) {
        Ok(i) => {
            println!("Startup successful!");
            i
        }
        Err(e) => {
            eprintln!("Startup error: {} \n exiting", e);
            return;
        }
    };

    thread::spawn(move || {
        cli::run_cli(transmit);
    });

    for command in receive {
        if let Err(e) = match command {
            Command::Play => internal.play(),
            Command::Pause => internal.pause(),
            Command::PlayNew(song) => internal.play_new(song),
            Command::Stop => Err("Stop command not implemented".to_string()),
            Command::CurrentSong(sender) => match internal.current_song() {
                Ok(song) => sender
                    .send(song)
                    .map_err(|e| format!("Failed to send current song: {}", e)),
                Err(e) => Err(e),
            },
            Command::PlaylistLoad(name) => internal.playlist_load(&name),
            Command::PlaylistNew {
                name,
                external_type,
            } => internal.playlist_new(&name, external_type),
            Command::PlaylistAdd(song) => internal.playlist_add(song),
            Command::PlaylistRemove(index) => internal.playlist_remove(index),
            Command::PlaylistMoveSong { from, to } => internal.playlist_move_song(from, to),
            Command::PlaylistGetSongs(sender) => match internal.playlist_get_songs() {
                Ok(songs) => sender
                    .send(songs)
                    .map_err(|e| format!("Failed to send playlist songs: {}", e)),
                Err(e) => Err(e),
            },
            Command::PlaylistGetName(sender) => match internal.playlist_get_name() {
                Ok(name) => sender
                    .send(name)
                    .map_err(|e| format!("Failed to send playlist name: {}", e)),
                Err(e) => Err(e),
            },
            Command::PlaylistSetName(name) => internal.playlist_set_name(name.as_str()),
            Command::PlaylistGetSong { song, index } => match internal.playlist_get_song(index) {
                Ok(s) => song
                    .send(s)
                    .map_err(|e| format!("Failed to send playlist song: {}", e)),
                Err(e) => Err(e),
            },
            Command::QueueAdd(song) => internal.queue_add(song),
            Command::QueueRemove(index) => internal.queue_remove(index),
            Command::QueueList => internal.queue_list(),
            Command::QueueNext => internal.queue_next(),
            Command::QueuePlaylist(playlist) => internal.queue_playlist(&playlist),
            Command::QueueCurrentPlaylist => internal.queue_current_playlist(),
            Command::QueueGet(sender) => match internal.queue_get() {
                Ok(queue) => sender
                    .send(queue.clone())
                    .map_err(|e| format!("Failed to send queue: {}", e)),
                Err(e) => Err(e),
            },
            Command::Shutdown => break,
            Command::UpdateAutoskip => internal.update_autoskip(),
        } {
            cli::invalid_input(e);
        }
    }

    loop {
        match shutdown(&internal) {
            Ok(_) => {
                println!("Shutdown successfull \n exiting");
                break;
            }
            Err(e) => eprintln!(
                "Shutdown error: {} \n trying again press ^C to force exit",
                e
            ),
        }
    }
}

enum Command {
    Play,
    Pause,
    PlayNew(Song),
    Stop,
    CurrentSong(mpsc::Sender<Song>),
    PlaylistLoad(String),
    PlaylistNew {
        name: String,
        external_type: Option<ExternalType>,
    },
    PlaylistAdd(Song),
    PlaylistRemove(usize),
    PlaylistMoveSong {
        from: usize,
        to: usize,
    },
    PlaylistGetSongs(mpsc::Sender<Vec<Song>>),
    PlaylistGetName(mpsc::Sender<String>),
    PlaylistSetName(String),
    PlaylistGetSong {
        song: mpsc::Sender<Song>,
        index: usize,
    },
    QueueAdd(Song),
    QueueRemove(usize),
    QueueList,
    QueueNext,
    QueuePlaylist(Playlist),
    QueueCurrentPlaylist,
    QueueGet(mpsc::Sender<Queue>),
    Shutdown,
    UpdateAutoskip,
}