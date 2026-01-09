use ui::cli;
use external::local;

mod external;
mod ui;
mod internal;

fn main () {
    let mut local = local::new();

    cli::run_cli(&mut local);

}