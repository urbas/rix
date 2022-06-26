use clap::Command;
use colored::*;
use rix::cmd;

fn main() {
    let mut cmd = Command::new("rix")
        .version("0.0.1")
        .about("Rix is another nix.");

    let subcommands = &[
        &cmd::build_derivation::cmd(),
        &cmd::hash::cmd(),
        &cmd::show_derivation::cmd(),
    ];

    for subcommand in subcommands {
        cmd = cmd.subcommand((subcommand.cmd)(Command::new(subcommand.name)));
    }

    if let Err(error) = dispatch_cmd(&cmd.get_matches(), subcommands) {
        eprintln!("{}: {}", "error".red(), error);
        std::process::exit(1);
    }
}

fn dispatch_cmd(
    parsed_args: &clap::ArgMatches,
    subcommands: &[&cmd::RixSubCommand],
) -> Result<(), String> {
    for subcommand in subcommands {
        if let Some(subcommand_args) = parsed_args.subcommand_matches(subcommand.name) {
            return (subcommand.handler)(subcommand_args);
        }
    }
    Err("operation not supported".to_owned())
}
