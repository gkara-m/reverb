use std::{sync::mpsc::{self, Sender}, time::Duration};

use crate::{Command, MAIN_SENDER, external::external::ExternalType, failure::failure::{Failure, FailureType}, internal::{playlist::Playlist, song::Song}};


pub(super) fn play() -> Result<(), Failure> {
    MAIN_SENDER.get().unwrap().clone().send(Command::Play)
    .map_err(|e| Failure::from((e.into(), FailureType::Fetal)))
}

pub(super) fn pause() -> Result<(), Failure> {
    MAIN_SENDER.get().unwrap().clone().send(Command::Pause)
    .map_err(|e| Failure::from((e.into(), FailureType::Fetal)))
}

pub(super) fn is_song_playing() -> Result<bool, Failure> {
    let (tx, rx) = mpsc::channel();
    MAIN_SENDER.get().unwrap().clone().send(Command::IsSongPlaying(tx))
    .map_err(|e| Failure::from((e.into(), FailureType::Fetal)))?;
    rx.recv()
    .map_err(|e| Failure::from((e.into(), FailureType::Fetal)))
}

pub(super) fn queue_get_songs() -> Result<Vec<Song>, Failure> {
    let (tx, rx) = mpsc::channel();
    MAIN_SENDER.get().unwrap().clone().send(Command::QueueGetSongs(tx))
    .map_err(|e| Failure::from((e.into(), FailureType::Fetal)))?;
    rx.recv()
    .map_err(|e| Failure::from((e.into(), FailureType::Fetal)))
}

pub(super) fn playlist_get_name() -> Result<String, Failure> {
    let (tx, rx) = mpsc::channel();
    MAIN_SENDER.get().unwrap().clone().send(Command::PlaylistGetName(tx))
    .map_err(|e| Failure::from((e.into(), FailureType::Fetal)))?;
    rx.recv()
    .map_err(|e| Failure::from((e.into(), FailureType::Fetal)))
}

pub(super) fn playlist_get_songs() -> Result<Vec<Song>, Failure> {
    let (tx, rx) = mpsc::channel();
    MAIN_SENDER.get().unwrap().clone().send(Command::PlaylistGetSongs(tx))
    .map_err(|e| Failure::from((e.into(), FailureType::Fetal)))?;
    rx.recv()
    .map_err(|e| Failure::from((e.into(), FailureType::Fetal)))
}

pub(super) fn queue_next() -> Result<(), Failure> {
    MAIN_SENDER.get().unwrap().send(Command::QueueNext)
    .map_err(|e| Failure::from((e.into(), FailureType::Fetal)))
}

pub(super) fn current_song() -> Result<Song, Failure> {
    let (tx, rx) = mpsc::channel();
    MAIN_SENDER.get().unwrap().clone().send(Command::CurrentSong(tx))
    .map_err(|e| Failure::from((e.into(), FailureType::Fetal)))?;
    rx.recv()
    .map_err(|e| Failure::from((e.into(), FailureType::Fetal)))
}

pub(super) fn play_new(song: Song) -> Result<(), Failure> {
    MAIN_SENDER.get().unwrap().clone().send(Command::PlayNew(song))
    .map_err(|e| Failure::from((e.into(), FailureType::Fetal)))
}

pub(super) fn queue_add(song: Song) -> Result<(), Failure> {
    MAIN_SENDER.get().unwrap().clone().send(Command::QueueAdd(song))
    .map_err(|e| Failure::from((e.into(), FailureType::Fetal)))
}

pub(super) fn queue_remove(song_index: usize) -> Result<(), Failure> {
    MAIN_SENDER.get().unwrap().clone().send(Command::QueueRemove(song_index))
    .map_err(|e| Failure::from((e.into(), FailureType::Fetal)))
}

pub(super) fn queue_playlist(playlist: Playlist) -> Result<(), Failure> {
    MAIN_SENDER.get().unwrap().clone().send(Command::QueuePlaylist(playlist))
    .map_err(|e| Failure::from((e.into(), FailureType::Fetal)))
}

pub(super) fn queue_current_playlist() -> Result<(), Failure> {
    MAIN_SENDER.get().unwrap().clone().send(Command::QueueCurrentPlaylist)
    .map_err(|e| Failure::from((e.into(), FailureType::Fetal)))
}

pub(super) fn playlist_add(song: Song) -> Result<(), Failure> {
    MAIN_SENDER.get().unwrap().clone().send(Command::PlaylistAdd(song))
    .map_err(|e| Failure::from((e.into(), FailureType::Fetal)))
}

pub(super) fn playlist_remove(index: usize) -> Result<(), Failure> {
    MAIN_SENDER.get().unwrap().clone().send(Command::PlaylistRemove(index))
    .map_err(|e| Failure::from((e.into(), FailureType::Fetal)))
}

pub(super) fn playlist_load(name: &str) -> Result<(), Failure> {
    MAIN_SENDER.get().unwrap().clone().send(Command::PlaylistLoad(name.to_string()))
    .map_err(|e| Failure::from((e.into(), FailureType::Fetal)))
}

pub(super) fn playlist_move_song(from: usize, to: usize) -> Result<(), Failure> {
    MAIN_SENDER.get().unwrap().clone().send(Command::PlaylistMoveSong { from, to })
    .map_err(|e| Failure::from((e.into(), FailureType::Fetal)))
}

pub(super) fn playlist_new(name: &str, external_type: Option<ExternalType>) -> Result<(), Failure> {
    MAIN_SENDER.get().unwrap().clone().send(Command::PlaylistNew { name: name.to_string(), external_type })
    .map_err(|e| Failure::from((e.into(), FailureType::Fetal)))
}

pub(super) fn playlist_get_song(index: usize) -> Result<Song, Failure> {
    let (tx, rx) = mpsc::channel();
    MAIN_SENDER.get().unwrap().clone().send(Command::PlaylistGetSong { song: tx, index })
    .map_err(|e| Failure::from((e.into(), FailureType::Fetal)))?;
    rx.recv()
    .map_err(|e| Failure::from((e.into(), FailureType::Fetal)))
}

pub(super) fn playlist_set_name(name: &str) -> Result<(), Failure> {
    MAIN_SENDER.get().unwrap().clone().send(Command::PlaylistSetName(name.to_string()))
    .map_err(|e| Failure::from((e.into(), FailureType::Fetal)))
}

pub(super) fn playlist_copy_to(name: &str) -> Result<(), Failure> {
    MAIN_SENDER.get().unwrap().clone().send(Command::PlaylistCopyTo(name.to_string()))
    .map_err(|e| Failure::from((e.into(), FailureType::Fetal)))
}

pub(super) fn playlist_add_playlist(playlist_name: &str) -> Result<(), Failure> {
    MAIN_SENDER.get().unwrap().clone().send(Command::PlaylistAddPlaylist(playlist_name.to_string()))
    .map_err(|e| Failure::from((e.into(), FailureType::Fetal)))
}

pub(super) fn playlist_clear() -> Result<(), Failure> {
    MAIN_SENDER.get().unwrap().clone().send(Command::PlaylistClear)
    .map_err(|e| Failure::from((e.into(), FailureType::Fetal)))
}

pub(super) fn shutdown() -> Result<(), Failure> {
    MAIN_SENDER.get().unwrap().clone().send(Command::Shutdown)
    .map_err(|e| Failure::from((e.into(), FailureType::Fetal)))
}

pub(super) fn song_duration() -> Result<Duration, Failure> {
    let (tx, rx) = mpsc::channel();
    MAIN_SENDER.get().unwrap().clone().send(Command::SongDuration(tx))
    .map_err(|e| Failure::from((e.into(), FailureType::Fetal)))?;
    match rx.recv() {
        Ok(progress) => progress,
        Err(e) => Err(Failure::from((e.into(), FailureType::Fetal))),
    }
}

pub(super) fn song_duration_gone() -> Result<Duration, Failure> {
    let (tx, rx) = mpsc::channel();
    MAIN_SENDER.get().unwrap().clone().send(Command::SongDurationGone(tx))
    .map_err(|e| Failure::from((e.into(), FailureType::Fetal)))?;
    match rx.recv() {
        Ok(progress) => progress,
        Err(e) => Err(Failure::from((e.into(), FailureType::Fetal))),
    }
}

pub(super) fn queue_shuffle() -> Result<(), Failure> {
    MAIN_SENDER.get().unwrap().clone().send(Command::QueueShuffle)
    .map_err(|e| Failure::from((e.into(), FailureType::Fetal)))
}

pub(super) fn queue_clear() -> Result<(), Failure> {
    MAIN_SENDER.get().unwrap().clone().send(Command::QueueClear)
    .map_err(|e| Failure::from((e.into(), FailureType::Fetal)))
}

pub(super) fn connect_to_server() -> Result<(), Failure> {
    MAIN_SENDER.get().unwrap().clone().send(Command::ServerConnect)
    .map_err(|e| Failure::from((e.into(), FailureType::Fetal)))
}

pub(super) fn send_message_to_server(message: &str) -> Result<(), Failure> {
    MAIN_SENDER.get().unwrap().clone().send(Command::ServerSendMessage(message.to_string()))
    .map_err(|e| Failure::from((e.into(), FailureType::Fetal)))
}

pub(super) fn add_server(name: String, address: String, certificate_path: String) -> Result<(), Failure> {
    MAIN_SENDER.get().unwrap().clone().send(Command::ServerAdd(name, address, certificate_path))
    .map_err(|e| Failure::from((e.into(), FailureType::Fetal)))
}