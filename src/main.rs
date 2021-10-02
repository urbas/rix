mod cmd_show_derivations;
mod parsers;

use clap::App;
use colored::*;

use crate::cmd_show_derivations::cmd_show_derivations;

fn main() {
    App::new("rix")
        .version("0.0.1")
        .about("Rix is another nix.")
        .subcommand(cmd_show_derivations())
        .get_matches();

    eprintln!("{}: operation not supported", "error".red());
    std::process::exit(1);
}
