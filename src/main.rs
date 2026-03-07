use std::{sync::mpsc, time::Duration};
use std::thread;

use anyhow::anyhow;
use internal::
    song::Song
;
use once_cell::sync::OnceCell;
use ui::cli::cli;

use crate::{
    config::startup_shutdown::{shutdown, startup}, 
    external::external::ExternalType, 
    failure::failure::{Failure, FailureType}, 
    internal::playlist::Playlist, 
    ui::cli::cli::print_failure
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
    //clear terminal first without clearing scrollback buffer
    let (_, height) = match crossterm::terminal::size() {
        Ok((width, height)) => (width, height),
        Err(e) => {
            print_failure(Failure::from((e.into(), FailureType::Warning)));
            (80, 24)
        }
    };
    for _ in 0..height {
        println!()
    }



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
        cli::run_cli(transmit, 100);
    });

    for command in receive {
        match match command {
            Command::Play => internal.play(),
            Command::Pause => internal.pause(),
            Command::IsSongPlaying(sender) => match internal.is_song_playing() {
                Ok(is_playing) => sender
                    .send(is_playing)
                    .map_err(|e| Failure::from((e.into(), FailureType::Warning))),
                Err(e) => Err(e),
            },
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
            Command::PlaylistCopyTo(name) => internal.playlist_copy_to(name.as_str()),
            Command::PlaylistAddPlaylist(playlist_name) => internal.playlist_add_playlist(playlist_name.as_str()),
            Command::PlaylistClear => internal.playlist_clear(),
            Command::QueueAdd(song) => {internal.queue_add(song); Ok(())},
            Command::QueueRemove(index) => internal.queue_remove(index),
            Command::QueueNext => internal.queue_next(),
            Command::QueuePlaylist(playlist) => {internal.queue_playlist(&playlist); Ok(())},
            Command::QueueCurrentPlaylist => {internal.queue_current_playlist(); Ok(())},
            Command::QueueGetSongs(sender) => sender
                    .send(internal.queue_get_songs())
                    .map_err(|e| Failure::from((e.into(), FailureType::Warning))),
            Command::QueueClear => {internal.queue_clear(); Ok(())},
            Command::Shutdown => break,
            Command::UpdateAutoskip => internal.update_autoskip(),
            Command::SongDuration(sender) => sender
                    .send(internal.song_duration())
                    .map_err(|e| Failure::from((e.into(), FailureType::Warning))),
            Command::SongDurationGone(sender) => sender
                    .send(internal.song_duration_gone())
                    .map_err(|e| Failure::from((e.into(), FailureType::Warning))),
        } {
            Ok(_) => {},
            Err(failure) => match failure {
                Failure::Fetal(_) => {print_failure(failure); break;},
                Failure::Warning(_) => print_failure(failure),
            },
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
    IsSongPlaying(mpsc::Sender<bool>),
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
    PlaylistCopyTo (String),
    PlaylistAddPlaylist (String),
    PlaylistClear,
    QueueAdd(Song),
    QueueRemove(usize),
    QueueNext,
    QueuePlaylist(Playlist),
    QueueCurrentPlaylist,
    QueueGetSongs(mpsc::Sender<Vec<Song>>),
    QueueClear,
    Shutdown,
    UpdateAutoskip,
    SongDuration(mpsc::Sender<Result<Duration, Failure>>),
    SongDurationGone(mpsc::Sender<Result<Duration, Failure>>),
}