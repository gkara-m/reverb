use crate::{
    Command,
    MAIN_SENDER,
    external::external::{self, External, ExternalRun, ExternalType},
    internal::{
        internet, playlist::Playlist, queue::Queue, song::{Song, SongInfo}
    },
};
use reverb_core::failure::failure::{Failure, FailureType};

use std::sync::mpsc;
use std::sync::mpsc::Sender;
use std::{thread, time::Duration};
use anyhow::anyhow;

pub struct Internal {
    current_external: ExternalRun,
    current_playlist: Playlist,
    queue: Queue,
    kill_sender: Sender<()>,
    server_connection: Option<internet::connection::InternetClient>,
}

impl Internal {
    pub fn new(
        queue: Queue,
        playlist: Playlist,
    ) -> Result<Self, Failure> {
        Ok(Internal {
            current_external: external::get_new_external_run_from_song(&queue.current_song()?)?,
            current_playlist: playlist,
            queue,
            kill_sender: mpsc::channel().0,
            server_connection: None,
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
        self.current_playlist.save()?;
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

    pub fn playlist_load(&mut self, playlist_name: &str) -> Result<(), Failure> {
        self.playlist_save()
    }

    //pub fn get_song_info(&self, song: &Song) -> Result<SongInfo, Failure> {
    //    self.current_external.get_song_info(song)
    //}
}

impl Internal {
    pub fn load_playlist(&mut self, playlist_name: &str) -> Result<(), Failure> {
        self.playlist_save()?;
        let playlist = Playlist::load(playlist_name)?;
        self.current_playlist = playlist;
        Ok(())
    }

    fn playlist_save(&self) -> Result<(), Failure> {
        self.current_playlist.save()
    }

    pub fn playlist_new(
        &mut self,
        name: &str,
        external_type: Option<ExternalType>,
    ) -> Result<(), Failure> {
        self.playlist_save()?;
        self.current_playlist = Playlist::new(name, external_type)?;
        self.playlist_save()
    }

    pub fn playlist_add(&mut self, song: Song) -> Result<(), Failure> {
        self.current_playlist.add(&song);
        self.playlist_save()
    }

    pub fn playlist_remove(&mut self, index: usize) -> Result<(), Failure> {
        self.current_playlist.remove(index)?;
        self.playlist_save()
    }

    pub fn playlist_move_song(&mut self, from: usize, to: usize) -> Result<(), Failure> {
        self.current_playlist.move_song(from, to)?;
        self.playlist_save()
    }

    pub fn playlist_get_songs(&self) -> Vec<Song> {
        self.current_playlist.get_songs()
    }

    pub fn playlist_get_name(&self) -> String {
        self.current_playlist.get_name()
    }

    pub fn playlist_set_name(&mut self, name: &str) -> Result<(), Failure> {
        self.current_playlist.set_name(name)?;
        self.playlist_save()
    }

    pub fn playlist_get_song(&self, index: usize) -> Result<Song, Failure> {
        self.current_playlist.get_song(index)
    }

    pub fn playlist_copy_to(&mut self, name: &str) -> Result<(), Failure> {
        let mut new_playlist = Playlist::new(name, self.current_playlist.get_type())?;
        for song in self.current_playlist.iter() {
            new_playlist.add(&song);
        }
        new_playlist.save()?;
        self.playlist_load(name)?;
        Ok(())
    }

    pub fn playlist_add_playlist(&mut self, playlist_name_from: &str) -> Result<(), Failure> {
        let playlist_from = Playlist::load(playlist_name_from)?;
        for song in playlist_from.iter() {
            self.current_playlist.add(&song);
        }
        self.playlist_save()?;
        Ok(())
    }

    pub fn playlist_clear(&mut self) -> Result<(), Failure> {
        self.current_playlist.clear();
        self.playlist_save()
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
                FailureType::Fetal => return Err(e),
            },
        };
        self.play_new(next_song)?;
        Ok(())
    }

    pub fn queue_playlist(&mut self, playlist: &Playlist) {
        self.queue.load_playlist(playlist);
    }

    pub fn queue_current_playlist(&mut self) {
        self.queue.load_playlist(&self.current_playlist);
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
                sender.send(Command::QueueNext).map_err(|e| Failure::from((e.into(), FailureType::Fetal)))?;
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

    pub fn send_message_to_server(&mut self, message: String) -> Result<(), Failure> {
        println!("Attempting to send message to server b: {}", message);
        if let Some(sc) = self.server_connection.as_mut() {
            println!("Attempting to send message to server a: {}", message);
            sc.send_message(message)
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