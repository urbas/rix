mod cmd;
mod derivations;
mod hashes;
mod parsers;

use clap::App;
use colored::*;

fn main() {
    let parsed_args = App::new("rix")
        .version("0.0.1")
        .about("Rix is another nix.")
        .subcommand(cmd::hash::cmd())
        .subcommand(cmd::show_derivation::cmd())
        .get_matches();

    if let Err(error) = dispatch_cmd(&parsed_args) {
        eprintln!("{}: {}", "error".red(), error);
        std::process::exit(1);
    }
}

fn dispatch_cmd(parsed_args: &clap::ArgMatches) -> Result<(), String> {
    if let Some(sub_command) = parsed_args.subcommand_matches(cmd::show_derivation::CMD_NAME) {
        cmd::show_derivation::handle_cmd(sub_command)
    } else if let Some(sub_command) = parsed_args.subcommand_matches(cmd::hash::CMD_NAME) {
        cmd::hash::handle_cmd(sub_command)
    } else {
        Err("operation not supported".to_owned())
    }
}
