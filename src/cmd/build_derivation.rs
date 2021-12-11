use crate::building;
use crate::cmd::{CmdHandler, CmdResult, RixSubCommand};
use crate::derivations;
use clap::{Arg, ArgMatches};
use std::path::PathBuf;
use tempfile::tempdir;

pub fn cmd<'a>() -> RixSubCommand<'a> {
    return RixSubCommand {
        name: "build-derivation",
        handler: &(handle_cmd as CmdHandler),
        cmd: |subcommand| {
            subcommand
            .about("builds the derivation assuming all dependencies are present in the store and won't be GC'd")
            .arg(Arg::with_name("DERIVATION").required(true).help(
                "The path of the derivation to build.",
            ))
        },
    };
}

pub fn handle_cmd(parsed_args: &ArgMatches) -> CmdResult {
    let derivation_path = parsed_args
        .value_of("DERIVATION")
        .ok_or("You must specify a derivation.")?;
    let derivation = derivations::load_derivation(derivation_path)?;
    let build_dir = create_build_dir()?;
    println!("{}", build_dir.to_str().unwrap());
    std::process::exit(building::build_derivation_sandboxed(
        &derivation,
        &build_dir,
    )?);
}

fn create_build_dir() -> Result<PathBuf, String> {
    tempdir()
        .map_err(|err| format!("Could not create the build directory. Error: {}", err))
        .map(|tmp_dir| tmp_dir.into_path())
}
