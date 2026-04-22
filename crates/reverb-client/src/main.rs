use std::thread;
use std::{sync::mpsc, time::Duration};

use internal::song::Song;
use once_cell::sync::OnceCell;
use reverb_core::network::Packet;
use ui::cli::cli;

use reverb_core::failure::failure::{Failure, FailureType};

use crate::{
    config::{config::Config, startup_shutdown::{shutdown, startup}}, 
    external::external::ExternalType, 
    ui::cli::cli::print_failure,
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
        internal.handle_command(command.clone());// TODO , clone is maybe to expensive?
        cli::handle_command(command.clone()); //TODO handle the errors aswell
        match match command {
            Command::Shutdown => break,
            Command::Failure(failure) => Err(failure),
            _ => Ok(()),
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

#[derive(Debug, Clone)]
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
    ServerGetOnlineUsers,
    ServerResponse(Packet),
    ServerSetEchoAvailability(bool),
    Failure(Failure),
}

