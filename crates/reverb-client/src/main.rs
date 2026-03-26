use std::thread;
use std::{sync::mpsc, time::Duration};

use internal::song::Song;
use once_cell::sync::OnceCell;
use ui::cli::cli;

use reverb_core::failure::failure::{Failure, FailureType};

use crate::{
    config::{config::Config, startup_shutdown::{shutdown, startup}}, 
    external::external::ExternalType, 
    internal::playlist::Playlist, 
    ui::cli::cli::print_failure
};

mod config;
mod external;
mod internal;
mod ui;

pub static CONFIG_FOLDER: &str = "configs/";
pub static CONFIG: OnceCell<Config> = OnceCell::new();

pub static DATA_FOLDER: OnceCell<String> = OnceCell::new();
pub static LOCAL_SONG_FOLDER_PATH: OnceCell<String> = OnceCell::new();

pub static MAIN_SENDER: OnceCell<mpsc::Sender<Command>> = OnceCell::new();

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
    MAIN_SENDER.set(transmit).unwrap();

    let mut internal = match startup() {
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
        match cli::run_cli(100) {
            Ok(_) => println!("CLI exited successfully"),
            Err(e) => MAIN_SENDER.get().unwrap().send(Command::Failure(e)).unwrap(),
        }
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
            Command::CurrentSong(sender) => match internal.current_song() {
                Ok(song) => sender.send(song)
                    .map_err(|e| Failure::from((e.into(), FailureType::Warning))),
                Err(e) => Err(e),
            },
            Command::PlaylistNew {
                name,
                external_type,
            } => internal.playlist_new(&name, external_type),
            Command::PlaylistAdd(playlist, song) => internal.playlist_add(&playlist, song),
            Command::PlaylistRemove(playlist, index) => internal.playlist_remove(&playlist, index),
            Command::PlaylistMoveSong {playlist, from, to } => internal.playlist_move_song(&playlist, from, to),
            Command::PlaylistGetSongs(playlist, sender) => {
                match internal.playlist_get_songs(&playlist) {
                    Ok(songs) => 
                        sender.send(songs)
                        .map_err(|e| Failure::from((e.into(), FailureType::Warning))),
                    Err(e) => Err(e),
                }
            },
            Command::PlaylistSetName(playlist, name) => internal.playlist_set_name(&playlist, &name),
            Command::PlaylistGetSong {playlist, song, index } => match internal.playlist_get_song(&playlist, index) {
                Ok(s) => 
                    song.send(s)
                    .map_err(|e| Failure::from((e.into(), FailureType::Warning))),
                Err(e) => Err(e),
            },
            Command::PlaylistCopyTo(from, to) => internal.playlist_copy_to(&from, &to),
            Command::PlaylistAddPlaylist(from, to) => {
                internal.playlist_add_playlist(&from, &to)
            }
            Command::PlaylistClear(playlist) => internal.playlist_clear(&playlist),
            Command::QueueShuffle => {
                internal.queue_shuffle();
                Ok(())
            }
            Command::QueueAdd(song) => {
                internal.queue_add(song);
                Ok(())
            }
            Command::QueueRemove(index) => internal.queue_remove(index),
            Command::QueueNext => internal.queue_next(),
            Command::QueuePlaylist(playlist) => internal.queue_playlist(&playlist),
            Command::QueueGetSongs(sender) => sender
                .send(internal.queue_get_songs())
                .map_err(|e| Failure::from((e.into(), FailureType::Warning))),
            Command::QueueClear => {
                internal.queue_clear();
                Ok(())
            }
            Command::Shutdown => break,
            Command::UpdateAutoskip => internal.update_autoskip(),
            Command::SongDuration(sender) => sender
                .send(internal.song_duration())
                .map_err(|e| Failure::from((e.into(), FailureType::Warning))),
            Command::SongDurationGone(sender) => sender
                    .send(internal.song_duration_gone())
                    .map_err(|e| Failure::from((e.into(), FailureType::Warning))),
            Command::ServerConnect => {internal.connect_to_server()},
            Command::ServerUpdateStatus(status) => {internal.update_server_connection_status(status); Ok(())},
            Command::ServerSendQuery(message) => {internal.send_query(message)},
            Command::ServerSendNotify(message) => {internal.send_notify(message)},
            Command::ServerAdd(name, address, certificate) => {internal.add_server(name, address, certificate)},
            Command::Failure(failure) => Err(failure),
        } {
            Ok(_) => {},
            Err(failure) => match failure.failure_type() {
                FailureType::Fatal => {print_failure(failure); break;},
                FailureType::Warning => print_failure(failure),
            },
        }
    }

    loop {
        match shutdown(&internal) {
            Ok(_) => {
                println!("Shutdown successfull \n exiting");
                break;
            }
            Err(e) => match e.failure_type() {
                FailureType::Fatal => {
                    eprintln!("Fatal Shutdown error: {} \n exiting immediately see logs for details",
                        e
                    );
                    break;
                }
                FailureType::Warning => {
                    eprintln!("Shutdown warning: {} trying again press ^C to force exit \r",
                        e
                    );
                }
            },
        }
    }
}

#[derive(Debug)]
pub enum Command {
    Play,
    Pause,
    IsSongPlaying(mpsc::Sender<bool>),
    PlayNew(Song),
    CurrentSong(mpsc::Sender<Song>),
    PlaylistNew {
        name: String,
        external_type: Option<ExternalType>,
    },
    PlaylistAdd(String, Song),
    PlaylistRemove(String, usize),
    PlaylistMoveSong {
        playlist: String,
        from: usize,
        to: usize,
    },
    PlaylistGetSongs(String, mpsc::Sender<Vec<Song>>),
    PlaylistSetName(String, String),
    PlaylistGetSong {
        playlist: String,
        song: mpsc::Sender<Song>,
        index: usize,
    },
    PlaylistCopyTo(String, String),
    PlaylistAddPlaylist(String, String),
    PlaylistClear(String),
    QueueShuffle,
    QueueAdd(Song),
    QueueRemove(usize),
    QueueNext,
    QueuePlaylist(String),
    QueueGetSongs(mpsc::Sender<Vec<Song>>),
    QueueClear,
    Shutdown,
    UpdateAutoskip,
    SongDuration(mpsc::Sender<Result<Duration, Failure>>),
    SongDurationGone(mpsc::Sender<Result<Duration, Failure>>),
    ServerAdd(String, String, String), // name, address, certificate path
    ServerConnect,
    ServerUpdateStatus(internal::internet::connection::ConnectionStatus),
    ServerSendQuery(String),
    ServerSendNotify(String),
    Failure(Failure),
}

