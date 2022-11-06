use clap::Command;
use rix::cmd;
use std::process::ExitCode;

fn main() -> ExitCode {
    let mut cmd = Command::new("rix")
        .version("0.0.1")
        .about("Rix is another nix.");

    let subcommands = &[
        &cmd::build_derivation::cmd(),
        &cmd::eval::cmd(),
        &cmd::hash::cmd(),
        &cmd::show_derivation::cmd(),
    ];

    for subcommand in subcommands {
        cmd = cmd.subcommand((subcommand.cmd)(Command::new(subcommand.name)));
    }

    dispatch_cmd(&cmd.get_matches(), subcommands)
}

fn dispatch_cmd(parsed_args: &clap::ArgMatches, subcommands: &[&cmd::RixSubCommand]) -> ExitCode {
    for subcommand in subcommands {
        if let Some(subcommand_args) = parsed_args.subcommand_matches(subcommand.name) {
            return (subcommand.handler)(subcommand_args)
                .map_or_else(|err| err, |_| ExitCode::SUCCESS);
        }
    }
    cmd::print_and_err("operation not supported")
}
