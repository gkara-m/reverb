use anyhow::anyhow;
use std::{
    io::{self, BufRead, Write},
    sync::mpsc::Sender,
};

use crossterm::{
    cursor, queue,
    style::Print,
    terminal::{self},
};
use reverb_core::failure::failure::{Failure, FailureType};
use crate::{
    Command,
    internal::song,
    ui::{cli::cli::print_failure, ui},
};

pub(super) fn run_ui(
    transmit: &Sender<Command>,
    input_tx: Sender<String>,
    update_interval: u64,
) -> std::thread::JoinHandle<()> {
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
                            println!(
                                "Automatic shutdown failed, please manually shutdown the application"
                            );
                        };
                        break;
                    }
                }
                Err(e) => {
                    print_failure(Failure::from((e.into(), FailureType::Fetal)));
                    if let Err(e) = main_transmit.send(Command::Shutdown) {
                        print_failure(Failure::from((e.into(), FailureType::Fetal)));
                        println!(
                            "Automatic shutdown failed, please manually shutdown the application"
                        );
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

            if let Err(e) = queue_progress_bar(width, (0, 1), &mut stdout) {
                print_failure(e);
            }

            if let Err(e) = queue_song_name(width, (0, 0), &mut stdout) {
                print_failure(e);
            }

            let half_width;
            let queue_width;
            let playlist_width;

            if width % 2 == 0 {
                half_width = width / 2;
                queue_width = half_width - 1;
                playlist_width = half_width;
            } else {
                half_width = (width + 1) / 2;
                queue_width = half_width - 1;
                playlist_width = half_width - 1;
            }

            if let Err(e) = queue_queue(
                (queue_width, height - 20),
                (0, 2),
                &mut stdout,
            ) {
                print_failure(e);
            }

            if let Err(e) = queue_draw_line(
                (half_width - 1, 2),
                (half_width - 1, height - 20),
                &mut stdout,
            ) {
                print_failure(e);
            }

            if let Err(e) = queue_playlist(
                (playlist_width, height - 20),
                (half_width, 2),
                &mut stdout,
                &main_transmit,
            ) {
                print_failure(e);
            }

            let _ = queue!(stdout, cursor::RestorePosition,);
            let _ = stdout.flush();
        }
    });

    renderer
}

fn queue_progress_bar(
    width: u16,
    position: (u16, u16),
    stdout: &mut std::io::Stdout,
) -> Result<(), Failure> {
    // get song progress
    let song_duration_gone = ui::song_duration_gone()?;
    let song_duration = ui::song_duration()?;


    // create progress bar
    let song_duration_text = format!(
        "{}:{:02}",
        (song_duration.as_secs() / 60) % 60,
        song_duration.as_secs() % 60
    );
    let song_progress_text = format!(
        "{}:{:02}",
        (song_duration_gone.as_secs() / 60) % 60,
        song_duration_gone.as_secs() % 60
    );
    let progress_space = width as usize
        - song_duration_text.chars().count()
        - song_progress_text.chars().count()
        - 2; // 2 for the brackets
    let progress_length = (song_duration_gone.as_secs_f32() / song_duration.as_secs_f32()
        * (progress_space as f32))
        .round() as usize;
    let progress_bar = song_progress_text
        + "["
        + &"=".repeat(progress_length)
        + &" ".repeat(progress_space as usize - progress_length)
        + "]"
        + song_duration_text.as_str();

    // render progress bar
    let _ = queue!(
        stdout,
        cursor::MoveTo(position.0, position.1),
        Print(progress_bar),
    );

    Ok(())
}

fn queue_song_name(
    width: u16,
    position: (u16, u16),
    stdout: &mut std::io::Stdout,
) -> Result<(), Failure> {
    let song = ui::current_song()?;
    let mut song_name = song.info.title;
    let song_name = if song_name.chars().count() as u16 > width {
        let mut truncated = song_name
            .chars()
            .take((width - 3) as usize)
            .collect::<String>();
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

fn queue_queue(
    size: (u16, u16),
    position: (u16, u16),
    stdout: &mut std::io::Stdout,
) -> Result<(), Failure> {
    if size.1 == 0 || size.0 <= 6 {
        return Ok(());
    }

    let mut queue = ui::queue_get_songs()?;
    queue.remove(0);
    let mut queue_text = vec![format!("Queue:{}", " ".repeat((size.0 - 6) as usize))];
    create_song_list(size, queue, &mut queue_text);

    for i in 0..size.1 {
        let _ = queue!(
            stdout,
            cursor::MoveTo(position.0, position.1 + i),
            Print(&queue_text[i as usize]),
        );
    }

    Ok(())
}

fn queue_playlist(
    size: (u16, u16),
    position: (u16, u16),
    stdout: &mut std::io::Stdout,
    transmit: &Sender<Command>,
) -> Result<(), Failure> {
    if size.1 == 0 || size.0 <= 8 {
        return Ok(());
    }

    let playlist = ui::playlist_get_songs()?;
    let mut playlist_text = Vec::new();
    let mut top_line = "Playlist:".to_string();
    push_width_aware(
        &mut top_line,
        ui::playlist_get_name()?.as_str(),
        "",
        "",
        &mut playlist_text,
        size.0,
    );
    playlist_text.push(format!(
        "{}{}",
        top_line,
        " ".repeat((size.0 - top_line.chars().count() as u16) as usize)
    ));
    create_song_list(size, playlist, &mut playlist_text);

    for i in 0..size.1 {
        let _ = queue!(
            stdout,
            cursor::MoveTo(position.0, position.1 + i),
            Print(&playlist_text[i as usize]),
        );
    }

    Ok(())
}

fn queue_draw_line(
    from: (u16, u16),
    to: (u16, u16),
    stdout: &mut std::io::Stdout,
) -> Result<(), Failure> {
    if from.0 == to.0 {
        // vertical line
        let x = from.0;
        for y in from.1..=to.1 {
            let _ = queue!(stdout, cursor::MoveTo(x, y), Print("|"),);
        }
        Ok(())
    } else if from.1 == to.1 {
        // horizontal line
        let y = from.1;
        for x in from.0..=to.0 {
            let _ = queue!(stdout, cursor::MoveTo(x, y), Print("-"),);
        }
        Ok(())
    } else if from.0 - to.0 == from.1 - to.1 {
        // diagonal line
        let mut x = from.0;
        let mut y = from.1;
        while x <= to.0 && y <= to.1 {
            let _ = queue!(stdout, cursor::MoveTo(x, y), Print("\\"),);
            x += 1;
            y += 1;
        }
        Ok(())
    } else if from.0 - to.0 == to.1 - from.1 {
        // diagonal line
        let mut x = from.0;
        let mut y = from.1;
        while x <= to.0 && y >= to.1 {
            let _ = queue!(stdout, cursor::MoveTo(x, y), Print("/"),);
            x += 1;
            y -= 1;
        }
        Ok(())
    } else {
        Err(Failure::from((
            anyhow!(
                "Cannot draw line from {:?} to {:?} because it is not horizontal, vertical or diagonal",
                from,
                to
            ),
            FailureType::Warning,
        )))
    }
}

fn create_song_list(size: (u16, u16), songs: Vec<song::Song>, text: &mut Vec<String>) {
    let mut queue_iter = songs.iter();
    let mut i = 1;
    while text.len() < size.1 as usize {
        match queue_iter.next() {
            Some(song) => {
                let mut song_text = String::new();
                // add index
                let index = format!("{}. ", i);
                push_width_aware(&mut song_text, &index, "", "", text, size.0);

                // add name
                let name = &song.info.title;
                push_width_aware(&mut song_text, name, "", "", text, size.0);

                // add artists
                let artists = &song.info.artists;
                push_width_aware(
                    &mut song_text,
                    artists[0].as_str(),
                    "  ",
                    " - ",
                    text,
                    size.0,
                );

                // add type
                let external_type = format!("({})", song.song_type.as_type());
                push_width_aware(
                    &mut song_text,
                    &external_type.to_string(),
                    "  ",
                    "  ",
                    text,
                    size.0,
                );

                if song_text.chars().count() > 0 {
                    song_text.push_str(
                        &" ".repeat((size.0 - song_text.chars().count() as u16) as usize),
                    );
                    text.push(song_text);
                }
                i += 1;
            }
            None => {
                text.push(" ".repeat(size.0 as usize));
            }
        }
    }
}

fn push_width_aware(
    string: &mut String,
    to_push: &str,
    new_line_start: &str,
    same_line_separator: &str,
    list: &mut Vec<String>,
    max_length: u16,
) {
    if string.chars().count() as u16
        + to_push.chars().count() as u16
        + same_line_separator.chars().count() as u16
        > max_length
    {
        string.push_str(&" ".repeat(max_length as usize - string.chars().count()));
        list.push(string.clone());
        string.clear();
        if to_push.chars().count() as u16 + new_line_start.chars().count() as u16 > max_length {
            let mut truncated = new_line_start.to_string();
            truncated.push_str(
                &to_push
                    .chars()
                    .take((max_length - new_line_start.chars().count() as u16 - 3) as usize)
                    .collect::<String>(),
            );
            truncated.push_str("...");
            list.push(truncated);
        } else {
            string.push_str(&format!("{}{}", new_line_start, to_push));
        }
    } else {
        string.push_str(&format!("{}{}", same_line_separator, to_push));
    }
}
