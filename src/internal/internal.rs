use crate::{
    Command,
    external::external::{self, External, ExternalRun, ExternalType},
    internal::{playlist::Playlist, queue::Queue, song::Song},
};

use std::{thread, time::Duration};
use std::
    sync::mpsc::Sender
;
use std::sync::mpsc;

pub struct Internal {
    current_external: ExternalRun,
    current_playlist: Playlist,
    queue: Queue,
    sender: std::sync::mpsc::Sender<crate::Command>,
    kill_sender: Sender<()>,
}

impl Internal {
    pub fn new(
        queue: Queue,
        playlist: Playlist,
        sender: std::sync::mpsc::Sender<crate::Command>,
    ) -> Result<Self, String> {
        Ok(Internal {
            current_external: external::get_new_external_run_from_song(&queue.current_song()?)?,
            current_playlist: playlist,
            queue,
            sender,
            kill_sender: mpsc::channel().0,
        })
    }

    pub fn play(&mut self) -> Result<(), String> {
        self.current_external.play()?;
        self.update_autoskip()
    }

    pub fn pause(&mut self) -> Result<(), String> {
        self.current_external.pause()
    }

    pub fn play_new(&mut self, song: Song) -> Result<(), String> {
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

    fn stop(&self) -> Result<(), String> {
        self.current_external.stop()
    }

    pub fn current_song(&self) -> Result<Song, String> {
        self.queue.current_song()
    }

    pub fn shutdown(&self) -> Result<(), String> {
        self.kill_autoskip();
        self.current_playlist.save()?;
        self.current_external.shutdown()?;
        Ok(())
    }

    pub fn is_song_playing(&self) -> Result<bool, String> {
        self.current_external.is_song_playing()
    }

    pub fn song_time_left(&self) -> Result<Duration, String> {
        self.current_external.time_left()
    }
}

impl Internal {
    pub fn playlist_load(&mut self, playlist_name: &str) -> Result<(), String> {
        self.playlist_save()?;
        let playlist = Playlist::load(playlist_name)?;
        self.current_playlist = playlist;
        Ok(())
    }

    fn playlist_save(&self) -> Result<(), String> {
        self.current_playlist.save()
    }

    pub fn playlist_new(
        &mut self,
        name: &str,
        external_type: Option<ExternalType>,
    ) -> Result<(), String> {
        self.playlist_save()?;
        self.current_playlist = Playlist::new(name, external_type)?;
        self.playlist_save()
    }

    pub fn playlist_add(&mut self, song: Song) -> Result<(), String> {
        self.current_playlist.add(&song)?;
        self.playlist_save()
    }

    pub fn playlist_remove(&mut self, index: usize) -> Result<(), String> {
        self.current_playlist.remove(index)?;
        self.playlist_save()
    }

    pub fn playlist_move_song(&mut self, from: usize, to: usize) -> Result<(), String> {
        self.current_playlist.move_song(from, to)?;
        self.playlist_save()
    }

    pub fn playlist_get_songs(&self) -> Result<Vec<Song>, String> {
        self.current_playlist.get_songs()
    }

    pub fn playlist_get_name(&self) -> Result<String, String> {
        self.current_playlist.get_name()
    }

    pub fn playlist_set_name(&mut self, name: &str) -> Result<(), String> {
        self.current_playlist.set_name(name)?;
        self.playlist_save()
    }

    pub fn playlist_get_song(&self, index: usize) -> Result<Song, String> {
        self.current_playlist.get_song(index)
    }
}

impl Internal {
    pub fn queue_add(&mut self, song: Song) -> Result<(), String> {
        self.queue.add(song)?;
        Ok(())
    }

    pub fn queue_remove(&mut self, song_index: usize) -> Result<(), String> {
        self.queue.remove(song_index)?;
        Ok(())
    }

    pub fn queue_list(&mut self) -> Result<(), String> {
        self.queue.list()?;
        Ok(())
    }

    pub fn queue_next(&mut self) -> Result<(), String> {
        let next_song = self.queue.next()?;
        self.play_new(next_song)?;
        Ok(())
    }

    pub fn queue_playlist(&mut self, playlist: &Playlist) -> Result<(), String> {
        self.queue.load_playlist(playlist)?;
        Ok(())
    }

    pub fn queue_current_playlist(&mut self) -> Result<(), String> {
        self.queue.load_playlist(&self.current_playlist)?;
        Ok(())
    }

    pub fn queue_get(&self) -> Result<&Queue, String> {
        Ok(&self.queue)
    }

    pub fn update_autoskip(&mut self) -> Result<(), String> {
        println!("Updating autoskip... ");
        match self.kill_autoskip() {
            Ok(_) => println!("Killed existing autoskip. "),
            Err(e) => println!("No existing autoskip to kill: {}. ", e), 
        };
        if self.is_song_playing()? {
            println!("Song is playing, setting up autoskip... ");
            let time_left = self.song_time_left()?;
            if time_left.is_zero() {
                println!("Time left is zero, skipping to next song... ");
                let sender = self.sender.clone();
                sender.send(Command::QueueNext).map_err(|e| format!("Failed to send QueueNext command: {}", e))?;
                println!("Sent QueueNext command. ");
                Ok(())
            } else {
                println!("Time left is {:?}, setting up autoskip... ", time_left);
                let sender = self.sender.clone();
                let (kill_sender, kill_receiver) = mpsc::channel();
                self.kill_sender = kill_sender;
                thread::spawn(move || {
                    println!("Autoskip thread started, sleeping for {:?}... ", time_left);
                    if let Ok(_) = kill_receiver.recv_timeout(time_left) {
                        println!("Received kill signal, not skipping to next song. ");
                        return;
                    }
                    if time_left < Duration::from_secs(1) {
                        println!("Time left was less than 1 second for {:?}... sending skip command", time_left);
                        match   sender.send(Command::QueueNext) {
                            Ok(_) => println!("Autoskip time elapsed, sent QueueNext command. "),
                            Err(e) => println!("Failed to send QueueNext command: {}", e),
                        };
                    } else {
                        match sender.send(Command::UpdateAutoskip) {
                            Ok(_) => println!("Autoskip time elapsed, sent UpdateAutoskip command. "),
                            Err(e) => println!("Failed to send UpdateAutoskip command: {}", e),
                        }
                    }
                });
                Ok(())
            }
        } else {
            println!("No song is playing, not setting up autoskip. ");
            Ok(())
        }
    }

    pub fn kill_autoskip(&self) -> Result<(), String> {
        self.kill_sender.send(()).map_err(|e| format!("Failed to kill autoskip: {}", e))?;
        Ok(())
    }
}
