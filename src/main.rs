use std::sync::mpsc;
use std::thread;

use anyhow::anyhow;
use internal::
    song::Song
;
use once_cell::sync::OnceCell;
use ui::cli;

use crate::{
    config::startup_shutdown::{shutdown, startup}, external::external::ExternalType, failure::failure::{Failure, FailureType}, internal::{playlist::Playlist, queue::Queue}
};

mod external;
mod internal;
mod ui;
mod config;
mod failure;

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
            Command::Stop => Err(Failure::from((anyhow!("Stop command not implemented"), FailureType::Warning))),
            Command::CurrentSong(sender) => match internal.current_song() {
                Ok(song) => sender
                    .send(song)
                    .map_err(|e| Failure::from((e.into(), FailureType::Warning))),
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
            Command::PlaylistGetSongs(sender) => sender
                    .send(internal.playlist_get_songs())
                    .map_err(|e| Failure::from((e.into(), FailureType::Warning))),
            Command::PlaylistGetName(sender) => sender
                    .send(internal.playlist_get_name())
                    .map_err(|e| Failure::from((e.into(), FailureType::Warning))),
            Command::PlaylistSetName(name) => internal.playlist_set_name(name.as_str()),
            Command::PlaylistGetSong { song, index } => match internal.playlist_get_song(index) {
                Ok(s) => song
                    .send(s)
                    .map_err(|e| Failure::from((e.into(), FailureType::Warning))),
                Err(e) => Err(e),
            },
            Command::QueueAdd(song) => {internal.queue_add(song); Ok(())},
            Command::QueueRemove(index) => internal.queue_remove(index),
            Command::QueueList => {internal.queue_list(); Ok(())},
            Command::QueueNext => internal.queue_next(),
            Command::QueuePlaylist(playlist) => {internal.queue_playlist(&playlist); Ok(())},
            Command::QueueCurrentPlaylist => {internal.queue_current_playlist(); Ok(())},
            Command::QueueGet(sender) => sender
                    .send(internal.queue_get().clone())
                    .map_err(|e| Failure::from((e.into(), FailureType::Warning))),
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
            Err(e) => match e {
                Failure::Fetal(e) => {
                    eprintln!("Fetal Shutdown error: {} \n exiting immediately see logs for details",
                        e
                    );
                    break;
                }
                Failure::Warning(e) => {
                    eprintln!("Shutdown warning: {} trying again press ^C to force exit \r",
                        e
                    );
                }
            }
        }
    }
}

#[derive(Debug)]
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