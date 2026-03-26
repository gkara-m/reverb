use std::{sync::mpsc::{self, Sender}, time::Duration};

use crate::{Command, MAIN_SENDER, external::external::ExternalType, internal::{playlist::Playlist, song::Song}};
use reverb_core::failure::failure::{Failure, FailureType};

pub(super) fn play() -> Result<(), Failure> {
    MAIN_SENDER.get().unwrap().clone().send(Command::Play)
    .map_err(|e| Failure::from((e.into(), "play", FailureType::Fatal)))
}

pub(super) fn pause() -> Result<(), Failure> {
    MAIN_SENDER.get().unwrap().clone().send(Command::Pause)
    .map_err(|e| Failure::from((e.into(), "pause", FailureType::Fatal)))
}

pub(super) fn is_song_playing() -> Result<bool, Failure> {
    let (tx, rx) = mpsc::channel();
    MAIN_SENDER.get().unwrap().clone().send(Command::IsSongPlaying(tx))
    .map_err(|e| Failure::from((e.into(), "is_song_playing", FailureType::Fatal)))?;
    rx.recv()
    .map_err(|e| Failure::from((e.into(), "is_song_playing", FailureType::Fatal)))
}

pub(super) fn queue_get_songs() -> Result<Vec<Song>, Failure> {
    let (tx, rx) = mpsc::channel();
    MAIN_SENDER.get().unwrap().clone().send(Command::QueueGetSongs(tx))
    .map_err(|e| Failure::from((e.into(), "queue_get_songs", FailureType::Fatal)))?;
    rx.recv()
    .map_err(|e| Failure::from((e.into(), "queue_get_songs", FailureType::Fatal)))
}

// pub(super) fn playlist_get_name() -> Result<String, Failure> {
//     let (tx, rx) = mpsc::channel();
//     MAIN_SENDER.get().unwrap().clone().send(Command::PlaylistGetName(tx))
//     .map_err(|e| Failure::from((e.into(), "playlist_get_name", FailureType::Fatal)))?;
//     rx.recv()
//     .map_err(|e| Failure::from((e.into(), "playlist_get_name", FailureType::Fatal)))
// }

pub(super) fn playlist_get_songs(playlist: &str) -> Result<Vec<Song>, Failure> {
    let (tx, rx) = mpsc::channel();
    MAIN_SENDER.get().unwrap().clone().send(Command::PlaylistGetSongs(playlist.to_string(), tx))
    .map_err(|e| Failure::from((e.into(), "playlist_get_songs", FailureType::Fatal)))?;
    rx.recv()
    .map_err(|e| Failure::from((e.into(), "playlist_get_songs", FailureType::Fatal)))
}

pub(super) fn playlist_get_song(playlist: &str, index: usize) -> Result<Song, Failure> {
    let (tx, rx) = mpsc::channel();
    MAIN_SENDER.get().unwrap().clone().send(Command::PlaylistGetSong { playlist: playlist.to_string(), song: tx, index })
    .map_err(|e| Failure::from((e.into(), "playlist_get_song", FailureType::Fatal)))?;
    rx.recv()
    .map_err(|e| Failure::from((e.into(), "playlist_get_song", FailureType::Fatal)))
}

pub(super) fn queue_next() -> Result<(), Failure> {
    MAIN_SENDER.get().unwrap().send(Command::QueueNext)
    .map_err(|e| Failure::from((e.into(), "queue_next", FailureType::Fatal)))
}

pub(super) fn current_song() -> Result<Song, Failure> {
    let (tx, rx) = mpsc::channel();
    MAIN_SENDER.get().unwrap().clone().send(Command::CurrentSong(tx))
    .map_err(|e| Failure::from((e.into(), "current_song", FailureType::Fatal)))?;
    rx.recv()
    .map_err(|e| Failure::from((e.into(), "current_song", FailureType::Fatal)))
}

pub(super) fn play_new(song: Song) -> Result<(), Failure> {
    MAIN_SENDER.get().unwrap().clone().send(Command::PlayNew(song))
    .map_err(|e| Failure::from((e.into(), "play_new", FailureType::Fatal)))
}

pub(super) fn queue_add(song: Song) -> Result<(), Failure> {
    MAIN_SENDER.get().unwrap().clone().send(Command::QueueAdd(song))
    .map_err(|e| Failure::from((e.into(), "queue_add", FailureType::Fatal)))
}

pub(super) fn queue_remove(song_index: usize) -> Result<(), Failure> {
    MAIN_SENDER.get().unwrap().clone().send(Command::QueueRemove(song_index))
    .map_err(|e| Failure::from((e.into(), "queue_remove", FailureType::Fatal)))
}

pub(super) fn queue_playlist(playlist: &str) -> Result<(), Failure> {
    MAIN_SENDER.get().unwrap().clone().send(Command::QueuePlaylist(playlist.to_string()))
    .map_err(|e| Failure::from((e.into(), "queue_playlist", FailureType::Fatal)))
}

// pub(super) fn queue_current_playlist() -> Result<(), Failure> {
//     MAIN_SENDER.get().unwrap().clone().send(Command::QueueCurrentPlaylist)
//     .map_err(|e| Failure::from((e.into(), "queue_current_playlist", FailureType::Fatal)))
// }

pub(super) fn playlist_add(playlist: &str, song: Song) -> Result<(), Failure> {
    MAIN_SENDER.get().unwrap().clone().send(Command::PlaylistAdd(playlist.to_string(), song))
    .map_err(|e| Failure::from((e.into(), "playlist_add", FailureType::Fatal)))
}

pub(super) fn playlist_remove(playlist: &str, index: usize) -> Result<(), Failure> {
    MAIN_SENDER.get().unwrap().clone().send(Command::PlaylistRemove(playlist.to_string(), index))
    .map_err(|e| Failure::from((e.into(), "playlist_remove", FailureType::Fatal)))
}

// pub(super) fn playlist_load(name: &str) -> Result<(), Failure> {
//     MAIN_SENDER.get().unwrap().clone().send(Command::PlaylistLoad(name.to_string()))
//     .map_err(|e| Failure::from((e.into(), "playlist_load", FailureType::Fatal)))
// }

pub(super) fn playlist_move_song(playlist: &str, from: usize, to: usize) -> Result<(), Failure> {
    MAIN_SENDER.get().unwrap().clone().send(Command::PlaylistMoveSong { playlist: playlist.to_string(), from, to })
    .map_err(|e| Failure::from((e.into(), "playlist_move_song", FailureType::Fatal)))
}

pub(super) fn playlist_new(name: &str, external_type: Option<ExternalType>) -> Result<(), Failure> {
    MAIN_SENDER.get().unwrap().clone().send(Command::PlaylistNew { name: name.to_string(), external_type })
    .map_err(|e| Failure::from((e.into(), "playlist_new", FailureType::Fatal)))
}

pub(super) fn playlist_set_name(playlist: &str, name: &str) -> Result<(), Failure> {
    MAIN_SENDER.get().unwrap().clone().send(Command::PlaylistSetName(playlist.to_string(), name.to_string()))
    .map_err(|e| Failure::from((e.into(), "playlist_set_name", FailureType::Fatal)))
}

pub(super) fn playlist_copy_to(from: &str, to: &str) -> Result<(), Failure> {
    MAIN_SENDER.get().unwrap().clone().send(Command::PlaylistCopyTo(from.to_string(), to.to_string()))
    .map_err(|e| Failure::from((e.into(), "playlist_copy_to", FailureType::Fatal)))
}

pub(super) fn playlist_add_playlist(from: &str, to: &str) -> Result<(), Failure> {
    MAIN_SENDER.get().unwrap().clone().send(Command::PlaylistAddPlaylist(from.to_string(), to.to_string()))
    .map_err(|e| Failure::from((e.into(), "playlist_add_playlist", FailureType::Fatal)))
}

pub(super) fn playlist_clear(playlist: &str) -> Result<(), Failure> {
    MAIN_SENDER.get().unwrap().clone().send(Command::PlaylistClear(playlist.to_string()))
    .map_err(|e| Failure::from((e.into(), "playlist_clear", FailureType::Fatal)))
}

pub(super) fn shutdown() -> Result<(), Failure> {
    MAIN_SENDER.get().unwrap().clone().send(Command::Shutdown)
    .map_err(|e| Failure::from((e.into(), "shutdown", FailureType::Fatal)))
}

pub(super) fn song_duration() -> Result<Duration, Failure> {
    let (tx, rx) = mpsc::channel();
    MAIN_SENDER.get().unwrap().clone().send(Command::SongDuration(tx))
    .map_err(|e| Failure::from((e.into(), "song_duration", FailureType::Fatal)))?;
    match rx.recv() {
        Ok(progress) => progress,
        Err(e) => Err(Failure::from((e.into(), "song_duration", FailureType::Fatal))),
    }
}

pub(super) fn song_duration_gone() -> Result<Duration, Failure> {
    let (tx, rx) = mpsc::channel();
    MAIN_SENDER.get().unwrap().clone().send(Command::SongDurationGone(tx))
    .map_err(|e| Failure::from((e.into(), "song_duration_gone", FailureType::Fatal)))?;
    match rx.recv() {
        Ok(progress) => progress,
        Err(e) => Err(Failure::from((e.into(), "song_duration_gone", FailureType::Fatal))),
    }
}

pub(super) fn queue_shuffle() -> Result<(), Failure> {
    MAIN_SENDER.get().unwrap().clone().send(Command::QueueShuffle)
    .map_err(|e| Failure::from((e.into(), "queue_shuffle", FailureType::Fatal)))
}

pub(super) fn queue_clear() -> Result<(), Failure> {
    MAIN_SENDER.get().unwrap().clone().send(Command::QueueClear)
    .map_err(|e| Failure::from((e.into(), "queue_clear", FailureType::Fatal)))
}

pub(super) fn connect_to_server() -> Result<(), Failure> {
    MAIN_SENDER.get().unwrap().clone().send(Command::ServerConnect)
    .map_err(|e| Failure::from((e.into(), "connect_to_server", FailureType::Fatal)))
}

pub(super) fn send_query(message: &str) -> Result<(), Failure> {
    MAIN_SENDER.get().unwrap().clone().send(Command::ServerSendQuery(message.to_string()))
    .map_err(|e| Failure::from((e.into(), "send_query", FailureType::Fatal)))
}

pub(super) fn send_notify(message: &str) -> Result<(), Failure> {
    MAIN_SENDER.get().unwrap().clone().send(Command::ServerSendNotify(message.to_string()))
    .map_err(|e| Failure::from((e.into(), "send_notify", FailureType::Fatal)))
}

pub(super) fn add_server(name: String, address: String, certificate_path: String) -> Result<(), Failure> {
    MAIN_SENDER.get().unwrap().clone().send(Command::ServerAdd(name, address, certificate_path))
    .map_err(|e| Failure::from((e.into(), "add_server", FailureType::Fatal)))
}
