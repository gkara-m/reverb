use reverb_core::network::{GetOnlineUsers, OnlineUsers, Packet, SetEchoAvailability};
use crate::{
    CONFIG, Command, MAIN_SENDER, external::external::{self, External, ExternalRun, ExternalType}, internal::{
        internet, playlist::Playlist, queue::Queue, song::Song
    }
};
use reverb_core::{failure::failure::{Failure, FailureType}};

use std::{num::NonZeroUsize, sync::mpsc};
use lru::LruCache;
use std::sync::mpsc::Sender;
use std::{thread, time::Duration};
use anyhow::anyhow;

pub struct Internal {
    current_external: ExternalRun,
    queue: Queue,
    autoskip_kill_sender: Sender<()>,
    server_connection: internet::connection::InternetClient,
    playlists: LruCache<String, Playlist>,
    ui_update_sender: Sender<()>,
}

// command handling
impl Internal {
    pub fn handle_command(&mut self, command: Command) -> Result<(), Failure> {
        match command {
                        Command::Play => self.play(),
            Command::Pause => self.pause(),
            Command::IsSongPlaying(sender) => match self.is_song_playing() {
                Ok(is_playing) => sender
                    .send(is_playing)
                    .map_err(|e| Failure::from((e.into(), FailureType::Warning))),
                Err(e) => Err(e),
            },
            Command::PlayNew(song) => self.play_new(song),
            Command::CurrentSong(sender) => match self.current_song() {
                Ok(song) => sender.send(song)
                    .map_err(|e| Failure::from((e.into(), FailureType::Warning))),
                Err(e) => Err(e),
            },
            Command::PlaylistNew {
                name,
                external_type,
            } => self.playlist_new(&name, external_type),
            Command::PlaylistAdd(playlist, song) => self.playlist_add(&playlist, song),
            Command::PlaylistRemove(playlist, index) => self.playlist_remove(&playlist, index),
            Command::PlaylistMoveSong {playlist, from, to } => self.playlist_move_song(&playlist, from, to),
            Command::PlaylistGetSongs(playlist, sender) => {
                match self.playlist_get_songs(&playlist) {
                    Ok(songs) => 
                        sender.send(songs)
                        .map_err(|e| Failure::from((e.into(), FailureType::Warning))),
                    Err(e) => Err(e),
                }
            },
            Command::PlaylistSetName(playlist, name) => self.playlist_set_name(&playlist, &name),
            Command::PlaylistGetSong {playlist, song, index } => match self.playlist_get_song(&playlist, index) {
                Ok(s) => 
                    song.send(s)
                    .map_err(|e| Failure::from((e.into(), FailureType::Warning))),
                Err(e) => Err(e),
            },
            Command::PlaylistCopyTo(from, to) => self.playlist_copy_to(&from, &to),
            Command::PlaylistAddPlaylist(from, to) => {
                self.playlist_add_playlist(&from, &to)
            }
            Command::PlaylistClear(playlist) => self.playlist_clear(&playlist),
            Command::QueueShuffle => {
                self.queue_shuffle();
                Ok(())
            }
            Command::QueueAdd(song) => {
                self.queue_add(song);
                Ok(())
            }
            Command::QueueRemove(index) => self.queue_remove(index),
            Command::QueueNext => self.queue_next(),
            Command::QueuePlaylist(playlist) => self.queue_playlist(&playlist),
            Command::QueueGetSongs(sender) => sender
                .send(self.queue_get_songs())
                .map_err(|e| Failure::from((e.into(), FailureType::Warning))),
            Command::QueueClear => {
                self.queue_clear();
                Ok(())
            }
            Command::UpdateAutoskip => self.update_autoskip(),
            Command::SongDuration(sender) => sender
            .send(self.song_duration())
            .map_err(|e| Failure::from((e.into(), FailureType::Warning))),
            Command::SongDurationGone(sender) => sender
            .send(self.song_duration_gone())
            .map_err(|e| Failure::from((e.into(), FailureType::Warning))),
            Command::ServerConnect => {self.connect_to_server()},
            Command::ServerUpdateStatus(status) => {self.server_update_connection_status(status); Ok(())},
            Command::ServerAdd(name, address, certificate) => {self.server_add(name, address, certificate)},
            Command::ServerGetOnlineUsers => self.server_get_online_users(),
            Command::ServerSetEchoAvailability(availability) => self.server_set_echo_availability(availability),
            _ => Ok(()),
        }
    }
}

// general functions for internal state management
impl Internal {
    pub fn new(
        queue: Queue,
    ) -> Result<Self, Failure> {
        Ok(Internal {
            current_external: external::get_new_external_run_from_song(&queue.current_song()?)?,
            queue,
            autoskip_kill_sender: mpsc::channel().0,
            server_connection: internet::connection::InternetClient::new(),
            playlists: LruCache::new(NonZeroUsize::new(10).unwrap()),
            ui_update_sender: mpsc::channel().0,
        })
    }

    pub fn play(&mut self) -> Result<(), Failure> {
        self.current_external.play()?;
        self.update_autoskip()
    }

    pub fn pause(&mut self) -> Result<(), Failure> {
        self.current_external.pause()
    }

    pub fn play_new(&mut self, song: Song) -> Result<(), Failure> {
        self.stop()?;
        if !song.song_type.same_type(&self.current_external) {
            self.current_external = external::get_new_external_run_from_song(&song)?;
        }
        self.queue.queued_songs[0] = song;
        self.current_external
            .play_new(&self.queue.queued_songs[0])?;
        self.update_autoskip()?;
        Ok(())
    }

    fn stop(&self) -> Result<(), Failure> {
        self.current_external.stop()
    }

    pub fn current_song(&self) -> Result<Song, Failure> {
        self.queue.current_song()
    }

    pub fn shutdown(&self) -> Result<(), Failure> {
        self.kill_autoskip();
        self.current_external.shutdown()?;
        Ok(())
    }

    pub fn is_song_playing(&self) -> Result<bool, Failure> {
        self.current_external.is_song_playing()
    }

    pub fn song_duration_gone(&self) -> Result<Duration, Failure> {
        self.current_external.song_duration_gone()
    }

    pub fn song_duration(&self) -> Result<Duration, Failure> {
        self.current_external.song_duration()
    }
}


// playlist management
impl Internal {
    pub fn playlist_new(&mut self, name: &str, external_type: Option<ExternalType>) -> Result<(), Failure> {
        let playlist = Playlist::new(name, external_type)?;
        playlist.save()?;
        self.playlists.put(name.into(), playlist);
        Ok(())
    }

    pub fn playlist_add(&mut self, playlist: &str, song: Song) -> Result<(), Failure> {
        let playlist = self.load_playlist(playlist)?;
        playlist.add(&song);
        playlist.save()?;
        Ok(())
    }

    pub fn playlist_remove(&mut self, playlist: &str, index: usize) -> Result<(), Failure> {
        let playlist = self.load_playlist(playlist)?;
        playlist.remove(index)?;
        playlist.save()
    }

    pub fn playlist_move_song(&mut self, playlist: &str, from: usize, to: usize) -> Result<(), Failure> {
        let playlist = self.load_playlist(playlist)?;
        playlist.move_song(from, to)?;
        playlist.save()
    }

    pub fn playlist_get_songs(&mut self, playlist: &str) -> Result<Vec<Song>, Failure> {
        let playlist = self.load_playlist(playlist)?;
        Ok(playlist.get_songs())
    }

    pub fn playlist_set_name(&mut self, playlist: &str, name: &str) -> Result<(), Failure> {
        let playlist = self.load_playlist(playlist)?;
        playlist.set_name(name)?;
        playlist.save()
    }

    pub fn playlist_get_song(&mut self, playlist: &str, index: usize) -> Result<Song, Failure> {
        let playlist = self.load_playlist(playlist)?;
        playlist.get_song(index)
    }

    pub fn playlist_copy_to(&mut self, playlist: &str, name: &str) -> Result<(), Failure> {
        let playlist = Playlist::load(playlist)?;
        let mut new_playlist = Playlist::new(name, playlist.get_type())?;
        for song in playlist.iter() {
            new_playlist.add(&song);
        }
        new_playlist.save()?;
        self.playlists.put(name.into(), new_playlist);
        Ok(())
    }

    pub fn playlist_add_playlist(&mut self, from: &str, to: &str) -> Result<(), Failure> {
        let playlist_from_songs = self.playlist_get_songs(from)?;
        let playlist_to = self.load_playlist(to)?;
        for song in playlist_from_songs {
            playlist_to.add(&song);
        }
        playlist_to.save()
    }

    pub fn playlist_clear(&mut self, playlist: &str) -> Result<(), Failure> {
        let playlist = self.load_playlist(playlist)?;
        playlist.clear();
        playlist.save()
    }

    fn load_playlist(&mut self, playlist: &str) -> Result<&mut Playlist, Failure> {
        if !self.playlists.contains(playlist) {
            let playlist = Playlist::load(playlist)?;
            self.playlists.put(playlist.get_name(), playlist);
        }
        Ok(self.playlists.get_mut(playlist).unwrap())
    }
}

// queue management
impl Internal {
    pub fn queue_add(&mut self, song: Song) {
        self.queue.add(song);
    }

    pub fn queue_remove(&mut self, song_index: usize) -> Result<(), Failure> {
        self.queue.remove(song_index)?;
        Ok(())
    }

    pub fn queue_get(&self) -> Queue {
        self.queue.clone()
    }

    pub fn queue_get_songs(&self) -> Vec<Song> {
        self.queue.get_songs()
    }

    pub fn queue_next(&mut self) -> Result<(), Failure> {
        let next_song = match self.queue.next() {
            Ok(song) => song,
            Err(e) => match e.failure_type() {
                FailureType::Warning => return Ok(()),
                FailureType::Fatal => return Err(e),
            },
        };
        self.play_new(next_song)?;
        Ok(())
    }

    pub fn queue_playlist(&mut self, playlist: &str) -> Result<(), Failure> {
        let playlist = Playlist::load(playlist)?;
        self.queue.load_playlist(&playlist);
        Ok(())
    }

    pub fn queue_clear(&mut self) {
        self.queue.clear();
    }

    pub fn queue_shuffle(&mut self) {
        self.queue.shuffle();
    }

    pub fn update_autoskip(&mut self) -> Result<(), Failure> {
        self.kill_autoskip();
        if self.is_song_playing()? {
            let time_left = self.song_duration()? - self.song_duration_gone()?;
            if time_left.is_zero() {
                let sender = MAIN_SENDER.get().unwrap().clone();
                sender.send(Command::QueueNext).map_err(|e| Failure::from((e.into(), FailureType::Fatal)))?;
                Ok(())
            } else {
                let sender = MAIN_SENDER.get().unwrap().clone();
                let (kill_sender, kill_receiver) = mpsc::channel();
                self.autoskip_kill_sender = kill_sender;
                thread::spawn(move || {
                    if let Ok(_) = kill_receiver.recv_timeout(time_left) {
                        return;
                    }
                    if time_left < Duration::from_secs(1) {
                        match sender.send(Command::QueueNext) {
                            Err(e) => println!(
                                "Failed to send QueueNext command queue may not skip automatically: {}",
                                e
                            ),
                            _ => (),
                        };
                    } else {
                        match sender.send(Command::UpdateAutoskip) {
                            Err(e) => println!(
                                "Failed to send UpdateAutoskip command queue may not skip automatically: {}",
                                e
                            ),
                            _ => (),
                        }
                    }
                });
                Ok(())
            }
        } else {
            Ok(())
        }
    }

    pub fn kill_autoskip(&self) {
        let _ = self.autoskip_kill_sender.send(());
    }
}

// server connection management
impl Internal {
    pub fn connect_to_server(&mut self) -> Result<(), Failure> {
        self.server_connection.connect()
    }

    pub fn server_get_online_users(&mut self) -> Result<(), Failure> {
        self.server_connection.send_message(Box::new(GetOnlineUsers{}))
    }

    pub fn server_update_connection_status(&mut self, status: internet::connection::ConnectionStatus) {
        self.server_connection.update_connection(status);
    }

    pub fn server_add(&mut self, name: String, address: String, certificate_path: String) -> Result<(), Failure> {
        crate::config::internet::ServerConfig::new(&address, &name, &certificate_path)?;
        Ok(())
    }

    pub fn server_set_echo_availability(&mut self, availability: bool) -> Result<(), Failure> {
        self.server_connection.send_message(Box::new(SetEchoAvailability(availability)))
    }
}