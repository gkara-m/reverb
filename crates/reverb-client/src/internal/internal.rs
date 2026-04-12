use reverb_core::network::{GetOnlineUsers, Packet};
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
    kill_sender: Sender<()>,
    server_connection: Option<internet::connection::InternetClient>,
    playlists: LruCache<String, Playlist>,
}

impl Internal {
    pub fn new(
        queue: Queue,
    ) -> Result<Self, Failure> {
        Ok(Internal {
            current_external: external::get_new_external_run_from_song(&queue.current_song()?)?,
            queue,
            kill_sender: mpsc::channel().0,
            server_connection: None,
            playlists: LruCache::new(NonZeroUsize::new(10).unwrap()),
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
                self.kill_sender = kill_sender;
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
        let _ = self.kill_sender.send(());
    }
}

impl Internal {
    pub fn connect_to_server(&mut self) -> Result<(), Failure> {
        self.server_connection = match self.server_connection {
            Some(_) => {
                return Err(Failure::from((anyhow!("Already connected to server"), FailureType::Warning)));
            },
            None => {
                let mut sc = internet::connection::InternetClient::new();
                sc.connect()?;
                Some(sc)
            },
        };
        Ok(())
    }

    pub fn scan_online_users(&mut self) -> Result<(), Failure> {
        if let Some(sc) = self.server_connection.as_mut() {
            sc.send_message(Box::new(GetOnlineUsers{}))
        } else {
            Err(Failure::from((anyhow!("Not connected to server"), FailureType::Warning)))
        }
    }

    pub fn update_server_connection_status(&mut self, status: internet::connection::ConnectionStatus) {
        if let Some(sc) = self.server_connection.as_mut() {
            sc.update_connection(status);
        }
    }

    pub fn add_server(&mut self, name: String, address: String, certificate_path: String) -> Result<(), Failure> {
        crate::config::internet::ServerConfig::new(&address, &name, &certificate_path)?;
        Ok(())
    }
}

