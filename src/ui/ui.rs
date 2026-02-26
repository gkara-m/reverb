use std::sync::mpsc::{self, Sender};


use crate::{Command, external::external::ExternalType, failure::failure::{Failure, FailureType}, internal::{playlist::Playlist, song::Song}};




pub(super) fn play(transmit: &Sender<Command>) -> Result<(), Failure> {
    transmit.clone().send(Command::Play)
    .map_err(|e| Failure::from((e.into(), FailureType::Fetal)))
}

pub(super) fn pause(transmit: &Sender<Command>) -> Result<(), Failure> {
    transmit.clone().send(Command::Pause)
    .map_err(|e| Failure::from((e.into(), FailureType::Fetal)))
}

pub(super) fn queue_get_songs(transmit: &Sender<Command>) -> Result<Vec<Song>, Failure> {
    let (tx, rx) = mpsc::channel();
    transmit.clone().send(Command::QueueGetSongs(tx))
    .map_err(|e| Failure::from((e.into(), FailureType::Fetal)))?;
    rx.recv()
    .map_err(|e| Failure::from((e.into(), FailureType::Fetal)))
}

pub(super) fn playlist_get_name(transmit: &Sender<Command>) -> Result<String, Failure> {
    let (tx, rx) = mpsc::channel();
    transmit.clone().send(Command::PlaylistGetName(tx))
    .map_err(|e| Failure::from((e.into(), FailureType::Fetal)))?;
    rx.recv()
    .map_err(|e| Failure::from((e.into(), FailureType::Fetal)))
}

pub(super) fn playlist_get_songs(transmit: &Sender<Command>) -> Result<Vec<Song>, Failure> {
    let (tx, rx) = mpsc::channel();
    transmit.clone().send(Command::PlaylistGetSongs(tx))
    .map_err(|e| Failure::from((e.into(), FailureType::Fetal)))?;
    rx.recv()
    .map_err(|e| Failure::from((e.into(), FailureType::Fetal)))
}

pub(super) fn queue_next(transmit: &Sender<Command>) -> Result<(), Failure> {
    transmit.send(Command::QueueNext)
    .map_err(|e| Failure::from((e.into(), FailureType::Fetal)))
}

pub(super) fn current_song(transmit: &Sender<Command>) -> Result<Song, Failure> {
    let (tx, rx) = mpsc::channel();
    transmit.clone().send(Command::CurrentSong(tx))
    .map_err(|e| Failure::from((e.into(), FailureType::Fetal)))?;
    rx.recv()
    .map_err(|e| Failure::from((e.into(), FailureType::Fetal)))
}

pub(super) fn play_new(transmit: &Sender<Command>, song: Song) -> Result<(), Failure> {
    transmit.clone().send(Command::PlayNew(song))
    .map_err(|e| Failure::from((e.into(), FailureType::Fetal)))
}

pub(super) fn queue_add(transmit: &Sender<Command>, song: Song) -> Result<(), Failure> {
    transmit.clone().send(Command::QueueAdd(song))
    .map_err(|e| Failure::from((e.into(), FailureType::Fetal)))
}

pub(super) fn queue_remove(transmit: &Sender<Command>, song_index: usize) -> Result<(), Failure> {
    transmit.clone().send(Command::QueueRemove(song_index))
    .map_err(|e| Failure::from((e.into(), FailureType::Fetal)))
}

pub(super) fn queue_playlist(transmit: &Sender<Command>, playlist: Playlist) -> Result<(), Failure> {
    transmit.clone().send(Command::QueuePlaylist(playlist))
    .map_err(|e| Failure::from((e.into(), FailureType::Fetal)))
}

pub(super) fn queue_current_playlist(transmit: &Sender<Command>) -> Result<(), Failure> {
    transmit.clone().send(Command::QueueCurrentPlaylist)
    .map_err(|e| Failure::from((e.into(), FailureType::Fetal)))
}

pub(super) fn playlist_add(transmit: &Sender<Command>, song: Song) -> Result<(), Failure> {
    transmit.clone().send(Command::PlaylistAdd(song))
    .map_err(|e| Failure::from((e.into(), FailureType::Fetal)))
}

pub(super) fn playlist_remove(transmit: &Sender<Command>, index: usize) -> Result<(), Failure> {
    transmit.clone().send(Command::PlaylistRemove(index))
    .map_err(|e| Failure::from((e.into(), FailureType::Fetal)))
}

pub(super) fn playlist_load(transmit: &Sender<Command>, name: &str) -> Result<(), Failure> {
    transmit.clone().send(Command::PlaylistLoad(name.to_string()))
    .map_err(|e| Failure::from((e.into(), FailureType::Fetal)))
}

pub(super) fn playlist_move_song(transmit: &Sender<Command>, from: usize, to: usize) -> Result<(), Failure> {
    transmit.clone().send(Command::PlaylistMoveSong { from, to })
    .map_err(|e| Failure::from((e.into(), FailureType::Fetal)))
}

pub(super) fn playlist_new(transmit: &Sender<Command>, name: &str, external_type: Option<ExternalType>) -> Result<(), Failure> {
    transmit.clone().send(Command::PlaylistNew { name: name.to_string(), external_type })
    .map_err(|e| Failure::from((e.into(), FailureType::Fetal)))
}

pub(super) fn playlist_get_song(transmit: &Sender<Command>, index: usize) -> Result<Song, Failure> {
    let (tx, rx) = mpsc::channel();
    transmit.clone().send(Command::PlaylistGetSong { song: tx, index })
    .map_err(|e| Failure::from((e.into(), FailureType::Fetal)))?;
    rx.recv()
    .map_err(|e| Failure::from((e.into(), FailureType::Fetal)))
}

pub(super) fn playlist_set_name(transmit: &Sender<Command>, name: &str) -> Result<(), Failure> {
    transmit.clone().send(Command::PlaylistSetName(name.to_string()))
    .map_err(|e| Failure::from((e.into(), FailureType::Fetal)))
}

pub(super) fn shutdown(transmit: &Sender<Command>) -> Result<(), Failure> {
    transmit.clone().send(Command::Shutdown)
    .map_err(|e| Failure::from((e.into(), FailureType::Fetal)))
}

pub(super) fn song_progress(transmit: &Sender<Command>) -> Result<f32, Failure> {
    let (tx, rx) = mpsc::channel();
    transmit.clone().send(Command::SongProgress(tx))
    .map_err(|e| Failure::from((e.into(), FailureType::Fetal)))?;
    match rx.recv() {
        Ok(progress) => progress,
        Err(e) => Err(Failure::from((e.into(), FailureType::Fetal))),
    }
}