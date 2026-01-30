use once_cell::sync::OnceCell;
use ui::cli;


mod external;
mod ui;
mod internal;
mod config;

pub static CONFIG_FOLDER: &str = "configs/";

pub static DATA_FOLDER: OnceCell<String> = OnceCell::new();
pub static LOCAL_SONG_FOLDER_PATH: OnceCell<String> = OnceCell::new();

fn main () {


    let mut internal = match config::startup_shutdown::startup() {
        Ok(i) => {
            println!("Startup successful!");
            i
        }
        Err(e) => {
            eprintln!("Startup error: {} \n exiting", e);
            return;
        }
    };

    cli::run_cli(&mut internal);

    loop {
        match config::startup_shutdown::shutdown(&internal) {
            Ok(_) => {println!("Shutdown successfull \n exiting");
            break;
        },
            Err(e) => eprintln!("Shutdown error: {} \n trying again press ^C to force exit", e),
        }
    }
}