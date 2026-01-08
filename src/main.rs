use ui::cli;
use external::{external::{External, ExternalType::LOCAL}, local};
use internal::song::Song;

mod external;
mod ui;
mod internal;

fn main () {
    let file_path = "sample/sf.mp3";
    let mut local = local::Local::new();

    cli::run_cli(&mut local);

}