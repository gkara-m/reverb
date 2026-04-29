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
    Command, MAIN_SENDER, internal::song, ui::{cli::cli::print_failure, ui}
};

pub(super) fn run_ui(
    input_tx: Sender<String>,
    update_interval: u64,
) -> std::thread::JoinHandle<()> {
    std::thread::spawn(move || {
        let stdin = io::stdin();
        for line in stdin.lock().lines() {
            match line {
                Ok(line) => {
                    if let Err(e) = input_tx.send(line) {
                        print_failure(Failure::from((e.into(), FailureType::Fatal)));
                        if let Err(e) = MAIN_SENDER.get().unwrap().clone().send(Command::Shutdown) {
                            print_failure(Failure::from((e.into(), FailureType::Fatal)));
                            println!(
                                "Automatic shutdown failed, please manually shutdown the application"
                            );
                        };
                        break;
                    }
                }
                Err(e) => {
                    print_failure(Failure::from((e.into(), FailureType::Fatal)));
                    if let Err(e) = MAIN_SENDER.get().unwrap().clone().send(Command::Shutdown) {
                        print_failure(Failure::from((e.into(), FailureType::Fatal)));
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


            let usable = width.saturating_sub(2);
            let third_width = usable / 3;

            // Left section
            if let Err(e) = queue_queue(
                (third_width, height - 20),
                (0, 2),
                &mut stdout,
            ) {
                print_failure(e);
            }

            // First vertical line
            if let Err(e) = queue_draw_line(
                (third_width, 2),
                (third_width, height - 20),
                &mut stdout,
            ) {
                print_failure(e);
            }

            // Middle section
            if let Err(e) = queue_playlist(
                (third_width, height - 20),
                (third_width + 1, 2),
                &mut stdout,
            ) {
                print_failure(e);
            }

            // Second vertical line
            if let Err(e) = queue_draw_line(
                (third_width * 2 + 1, 2),
                (third_width * 2 + 1, height - 20),
                &mut stdout,
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
) -> Result<(), Failure> {
    if size.1 == 0 || size.0 <= 8 {
        return Ok(());
    }

    let playlist = ui::playlist_get_songs(crate::ui::cli::cli::PLAYLIST.lock().unwrap().as_str())?;
    let mut playlist_text = Vec::new();
    let mut top_line = "Playlist:".to_string();
    push_width_aware(
        &mut top_line,
        crate::ui::cli::cli::PLAYLIST.lock().unwrap().as_str(),
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

// Appends a string to a line buffer with width-aware formatting for terminal UIs.
//
// Behavior:
// - If `to_push` fits on the current line (with `same_line_separator`), it is appended with the separator.
// - If it doesn't fit, the current line is padded to `max_length`, pushed to `list`, and the buffer is cleared.
// - If `to_push` fits on a new line (with `new_line_start`), it starts a new line with that prefix.
// - If it still doesn't fit, it is truncated with ellipsis and pushed as a new line.
//
// this edits in place the list passed in with the lines to print and the string passed in with the last line built (so that it can be used for the next call to this function)
// after last call the string should be pushed to the list and cleared
//
// This is used to build lines for terminal output, ensuring no line exceeds `max_length`.
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
    } else if string.chars().count() == 0 {
        string.push_str(&format!("{}", to_push));
    } else {
        string.push_str(&format!("{}{}", same_line_separator, to_push));
    }
}


// show text in right third of the terminal (defined as starting from (integer division) ((width-2)/3)*2+2 and finishing at width-1) and starting from line 2 and finishing at height-20
pub fn show_text_in_right_third(text: &str) {
    let mut stdout = io::stdout();
    let (width, height) = match terminal::size() {
        Ok((width, height)) => (width, height),
        Err(e) => {
            print_failure(Failure::from((e.into(), FailureType::Warning)));
            (80, 24)
        }
    };

    let usable = width.saturating_sub(2);
    let third_width = usable / 3;
    let right_width = third_width + (usable % 3);
    let right_start = third_width * 2 + 2;

    let _ = queue!(
        stdout,
        cursor::SavePosition,
    );

    let mut current_line = 2;

    for in_line in text.split('\n') {
        let mut out_line = String::new();
        let mut out_lines = Vec::new();
        for word in in_line.split_whitespace() {
            push_width_aware(&mut out_line, word, "  ", " ", &mut out_lines, right_width);
        }
        if out_line.chars().count() > 0 {
            out_line.push_str(&" ".repeat(right_width as usize - out_line.chars().count() as usize));
            out_lines.push(out_line);
        }
        for out_line in out_lines.iter() {
            let _ = queue!(
                stdout,
                cursor::MoveTo(right_start, current_line as u16),
                Print(out_line),
            );
            current_line += 1;
            if current_line >= height - 20 {
                break;
            }
        }
    }
    for i in current_line..(height - 19) {
        let _ = queue!(
            stdout,
            cursor::MoveTo(right_start, i),
            Print(" ".repeat(right_width as usize)),
        );
    }
    let _ = queue!(
        stdout,
        cursor::RestorePosition,
    );

    let _ = stdout.flush();
}