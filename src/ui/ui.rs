use std::sync::mpsc::{self, Sender};

use crate::{Command, external::external::ExternalType, internal::{playlist::Playlist, song::Song}};




pub(super) fn play(transmit: &Sender<Command>) -> Result<(), String> {
    transmit.clone().send(Command::Play).map_err(|e| format!("Failed to send play command: {}", e))
}

pub(super) fn pause(transmit: &Sender<Command>) -> Result<(), String> {
    transmit.clone().send(Command::Pause).map_err(|e| format!("Failed to send pause command: {}", e))
}

pub(super) fn queue_list(transmit: &Sender<Command>) -> Result<(), String> {
    transmit.clone().send(Command::QueueList).map_err(|e| format!("Failed to send queue list command: {}", e))
}

pub(super) fn playlist_get_name(transmit: &Sender<Command>) -> Result<&String, String> {
    let (tx, rx) = mpsc::channel();
    transmit.clone().send(Command::PlaylistGetName(tx)).map_err(|e| format!("Failed to send playlist get name command: {}", e))?;
    let name= rx.recv().map_err(|e| format!("Failed to receive playlist name: {}", e))?;
    Ok(name)
}

pub(super) fn playlist_get_songs(transmit: &Sender<Command>) -> Result<&Vec<Song>, String> {
    let (tx, rx) = mpsc::channel();
    transmit.clone().send(Command::PlaylistGetSongs(tx)).map_err(|e| format!("Failed to send playlist get songs command: {}", e))?;
    let songs = rx.recv().map_err(|e| format!("Failed to receive playlist songs: {}", e))?;
    Ok(songs)
}

pub(super) fn queue_next(transmit: &Sender<Command>) -> Result<(), String> {
    transmit.send(Command::QueueNext).map_err(|e| format!("Failed to send queue next command: {}", e))
}

pub(super) fn current_song(transmit: &Sender<Command>) -> Result<Song, String> {
    let (tx, rx) = mpsc::channel();
    transmit.clone().send(Command::CurrentSong(tx)).map_err(|e| format!("Failed to send current song command: {}", e))?;
    let song = rx.recv().map_err(|e| format!("Failed to receive current song: {}", e))?;
    Ok(song)
}

pub(super) fn play_new(transmit: &Sender<Command>, song: Song) -> Result<(), String> {
    transmit.clone().send(Command::PlayNew(song)).map_err(|e| format!("Failed to send play new command: {}", e))
}

pub(super) fn queue_add(transmit: &Sender<Command>, song: Song) -> Result<(), String> {
    transmit.clone().send(Command::QueueAdd(song)).map_err(|e| format!("Failed to send queue add command: {}", e))
}

pub(super) fn queue_remove(transmit: &Sender<Command>, song_index: usize) -> Result<(), String> {
    transmit.clone().send(Command::QueueRemove(song_index)).map_err(|e| format!("Failed to send queue remove command: {}", e))
}

pub(super) fn queue_playlist(transmit: &Sender<Command>, playlist: &Playlist) -> Result<(), String> {
    transmit.clone().send(Command::QueuePlaylist(playlist)).map_err(|e| format!("Failed to send queue playlist command: {}", e))
}

pub(super) fn queue_current_playlist(transmit: &Sender<Command>) -> Result<(), String> {
    transmit.clone().send(Command::QueueCurrentPlaylist).map_err(|e| format!("Failed to send queue current playlist command: {}", e))
}

pub(super) fn playlist_add(transmit: &Sender<Command>, song: Song) -> Result<(), String> {
    transmit.clone().send(Command::PlaylistAdd(song)).map_err(|e| format!("Failed to send playlist add command: {}", e))
}

pub(super) fn playlist_remove(transmit: &Sender<Command>, index: usize) -> Result<(), String> {
    transmit.clone().send(Command::PlaylistRemove(index)).map_err(|e| format!("Failed to send playlist remove command: {}", e))
}

pub(super) fn playlist_load(transmit: &Sender<Command>, name: &str) -> Result<(), String> {
    transmit.clone().send(Command::PlaylistLoad(name)).map_err(|e| format!("Failed to send playlist load command: {}", e))
}

pub(super) fn playlist_move_song(transmit: &Sender<Command>, from: usize, to: usize) -> Result<(), String> {
    transmit.clone().send(Command::PlaylistMoveSong { from, to }).map_err(|e| format!("Failed to send playlist move song command: {}", e))
}

pub(super) fn playlist_new(transmit: &Sender<Command>, name: &str, external_type: Option<ExternalType>) -> Result<(), String> {
    transmit.clone().send(Command::PlaylistNew { name, external_type }).map_err(|e| format!("Failed to send playlist new command: {}", e))
}

pub(super) fn playlist_get_song(transmit: &Sender<Command>, index: usize) -> Result<&Song, String> {
    let (tx, rx) = mpsc::channel();
    transmit.clone().send(Command::PlaylistGetSong { song: tx, index }).map_err(|e| format!("Failed to send playlist get song command: {}", e))?;
    let song = rx.recv().map_err(|e| format!("Failed to receive playlist song: {}", e))?;
    Ok(song)
}

pub(super) fn playlist_set_name(transmit: &Sender<Command>, name: &str) -> Result<(), String> {
    transmit.clone().send(Command::PlaylistSetName(name)).map_err(|e| format!("Failed to send playlist set name command: {}", e))
}