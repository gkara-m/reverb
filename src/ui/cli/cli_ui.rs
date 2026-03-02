use std::{io::{self, BufRead, Write}, sync::mpsc::Sender};

use crossterm::{cursor, queue, style::Print, terminal::{self}};

use crate::{Command, failure::failure::{Failure, FailureType}, internal::song, ui::{cli::cli::print_failure, ui}};

pub(super) fn run_ui(transmit: &Sender<Command>, input_tx: Sender<String>, update_interval: u64) -> std::thread::JoinHandle<()> {
    let main_transmit = transmit.clone();
    std::thread::spawn(move || {
        let stdin = io::stdin();
        for line in stdin.lock().lines() {
            match line {
                Ok(line) => {
                    if let Err(e) = input_tx.send(line) {
                        print_failure(Failure::from((e.into(), FailureType::Fetal)));
                        if let Err(e) = main_transmit.send(Command::Shutdown) {
                            print_failure(Failure::from((e.into(), FailureType::Fetal)));
                            println!("Automatic shutdown failed, please manually shutdown the application");
                        };
                        break;
                    }
                }
                Err(e) => {
                    print_failure(Failure::from((e.into(), FailureType::Fetal)));
                    if let Err(e) = main_transmit.send(Command::Shutdown) {
                        print_failure(Failure::from((e.into(), FailureType::Fetal)));
                        println!("Automatic shutdown failed, please manually shutdown the application");
                    };
                    break;
                }
            }
        }
    });


    //renderer thread
    let main_transmit = transmit.clone();
    let renderer = std::thread::spawn(move || {
        let mut stdout = io::stdout();
        loop {
            std::thread::sleep(std::time::Duration::from_millis(update_interval));

            // check line length
            let (width, height) = match terminal::size() {
                Ok((width, height)) => (width, height),
                Err(e) => {
                    print_failure(Failure::from((e.into(), FailureType::Warning)));
                    (80, 24)
                }
            };

            //render
            let _ = queue!(stdout, cursor::SavePosition);

            if let Err(e) = queue_progress_bar(width, (0, 1), &mut stdout, &main_transmit) {
                print_failure(e);
            }

            if let Err(e) = queue_song_name(width, (0, 0), &mut stdout, &main_transmit) {
                print_failure(e);
            }

            if let Err(e) = queue_queue((width, height - 20), (0, 2), &mut stdout, &main_transmit) {
                print_failure(e);
            }

            let _ = queue!(stdout, cursor::RestorePosition,);            
            let _ = stdout.flush();
        }
    });

    renderer
}


fn queue_progress_bar(width: u16, position: (u16, u16), stdout: &mut std::io::Stdout, transmit: &Sender<Command>) -> Result<(), Failure> {
    // get song progress
    let song_duration_gone = ui::song_duration_gone(transmit)?;
    let song_duration = ui::song_duration(transmit)?;


    // create progress bar
    let song_duration_text = format!("{}:{:02}", (song_duration.as_secs() / 60) % 60, song_duration.as_secs() % 60);
    let song_progress_text = format!("{}:{:02}", (song_duration_gone.as_secs() / 60) % 60, song_duration_gone.as_secs() % 60);
    let progress_space = width as usize - song_duration_text.chars().count() - song_progress_text.chars().count() - 2; // 2 for the brackets
    let progress_length = (song_duration_gone.as_secs_f32() / song_duration.as_secs_f32() * (progress_space as f32)).round() as usize;
    let progress_bar = 
        song_progress_text +
        "[" + 
        &"=".repeat(progress_length) + 
        &" ".repeat(progress_space as usize - progress_length) + 
        "]" +
        song_duration_text.as_str()
        ;

    // render progress bar
    let _ = queue!(
        stdout,
        cursor::MoveTo(position.0, position.1),
        Print(progress_bar),
    );

    Ok(())
}

fn queue_song_name(width: u16, position: (u16, u16), stdout: &mut std::io::Stdout, transmit: &Sender<Command>) -> Result<(), Failure> {
    let song = ui::current_song(transmit)?;
    let mut song_name = song.info.title;
    let song_name = if song_name.chars().count() as u16 > width {
        let mut truncated = song_name.chars().take((width - 3) as usize).collect::<String>();
        truncated.push_str("...");
        truncated
    } else {
        song_name.push_str(&" ".repeat((width - song_name.chars().count() as u16) as usize));
        song_name
    };
    
    let _ = queue!(
        stdout,
        cursor::MoveTo(position.0, position.1),
        Print(song_name),
    );

    Ok(())
}


// create and queue the queue text, truncating as necessary to fit the width and height
// this looks 
fn queue_queue(size: (u16, u16), position: (u16, u16), stdout: &mut std::io::Stdout, transmit: &Sender<Command>) -> Result<(), Failure> {
    if size.1 == 0 || size.0 == 0 {
        return Ok(());
    }
    
    let queue = ui::queue_get_songs(transmit)?;
    let mut queue_iter = queue.iter();
    let mut queue_text = Vec::new();
    while queue_text.len() < size.1 as usize {
        match queue_iter.next() {
            Some(song) => {
                let mut song_text = String::new();

                // add name
                let name = &song.info.title;
                push_helper(&mut song_text, name, "", "", &mut queue_text, size.0);

                // add artist
                let artist = &song.info.artist;
                push_helper(&mut song_text, artist, "  ", " - ", &mut queue_text, size.0);


                // add type
                let external_type = format!("({})", song.song_type.as_type());
                push_helper(&mut song_text, &external_type.to_string(), "  ", "  ", &mut queue_text, size.0);

                if song_text.chars().count() > 0 {
                    queue_text.push(song_text);
                }
            },
            None => {
                queue_text.push(" ".to_string());
            },
        }
    }

    for i in 0..size.1 {
        if (queue_text[i as usize].chars().count() as u16) < size.0 {
            let spaces = size.0 - queue_text[i as usize].chars().count() as u16;
            queue_text[i as usize].push_str(&" ".repeat(spaces as usize));
        }
        let _ = queue!(
            stdout,
            cursor::MoveTo(position.0, position.1 + i),
            Print(&queue_text[i as usize]),
        );
    }

    Ok(())
}

fn push_helper(string: &mut String, to_push: &str, new_line_start: &str, same_line_separator: &str, list: &mut Vec<String>, max_length: u16) {
    if string.chars().count() as u16 + to_push.chars().count() as u16 + 3 > max_length {
        list.push(string.clone());
        string.clear();
        if to_push.chars().count() as u16 + new_line_start.chars().count() as u16 > max_length as u16 {
            let mut truncated = new_line_start.to_string();
            truncated.push_str(&to_push.chars().take((max_length - new_line_start.chars().count() as u16 - 3) as usize).collect::<String>());
            truncated.push_str("...");
            list.push(truncated);
        } else {
            string.push_str(&format!("{}{}", new_line_start, to_push));
        }
    } else {
        string.push_str(&format!("{}{}", same_line_separator, to_push));
    }
}