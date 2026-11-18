use std::{env, thread};
use std::{sync::mpsc, time::Duration};

use log::{error, info, warn};
use once_cell::sync::OnceCell;
use reverb_core::network::Packet;
use ui::cli::cli;

use reverb_core::failure::failure::{Failure, FailureType};

use crate::{
    config::{startup_shutdown::{shutdown, startup}}, 
    external::external::ExternalType, 
    ui::cli::cli::print_failure,
};

mod config;
mod external;
mod internal;
mod ui;

use internal::song::Song;


pub static MAIN_SENDER: OnceCell<mpsc::Sender<Command>> = OnceCell::new();

fn main() {
    unsafe {// safe since there are no other threads running yet
        env::set_var("RUST_BACKTRACE", "1");
        if env::var("RUST_LOG").is_err() {
            env::set_var("RUST_LOG", "trace"); // default log level if not set by user, currently set to trace for development
        }
    }

    // initialise the logger
    env_logger::init();

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
            info!("Startup successful");
            i
        }
        Err(e) => {
            error!("Startup error: {} - exiting", e);
            return;
        }
    };

    // start CLI thread
    thread::spawn(move || {
        info!("Starting CLI");
        match cli::run_cli(100, false) {
            Ok(_) => info!("CLI exited successfully"),
            Err(e) => MAIN_SENDER.get().unwrap().send(Command::Failure(e)).unwrap(),
        }
    });

    for command in receive {
        match internal.handle_command(command.clone()) {
            Ok(_) => {},
            Err(failure) => match failure.failure_type() {
                FailureType::Fatal => {print_failure(failure); break;},
                FailureType::Warning => print_failure(failure),
            },
        };
        match cli::handle_command(command.clone()) {
            Ok(_) => {},
            Err(failure) => match failure.failure_type() {
                FailureType::Fatal => {print_failure(failure); break;},
                FailureType::Warning => print_failure(failure),
            },
        };
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
                info!("Shutdown successful - exiting");
                break;
            }
            Err(e) => match e.failure_type() {
                FailureType::Fatal => {
                    error!("Fatal shutdown error: {} - exiting immediately", e);
                    break;
                }
                FailureType::Warning => {
                    warn!("Shutdown warning: {} - trying again; press Ctrl+C to force exit", e);
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

