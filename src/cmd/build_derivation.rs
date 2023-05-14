use crate::building::{build_derivation_sandboxed, BuildConfig};
use crate::cmd::{to_cmd_err, RixSubCommand};
use crate::derivations;
use crate::store::nix_delegation_store::NixDelegationStore;
use clap::{Arg, ArgAction, ArgMatches};
use std::fs::File;
use std::path::PathBuf;
use tempfile::tempdir;

pub fn cmd() -> RixSubCommand {
    RixSubCommand {
        name: "build-derivation",
        handler: |args| to_cmd_err(handle_cmd(args)),
        cmd: |subcommand| {
            subcommand
            .about("builds the derivation assuming all dependencies are present in the store and won't be GC'd")
            .arg(Arg::new("DERIVATION").required(true).help(
                "The path of the derivation to build.",
            ))
            .arg(Arg::new("build-dir").long("build-dir").action(ArgAction::Set).help("The directory in which to run the build process."))
            .arg(Arg::new("stdout").long("stdout").action(ArgAction::Set).help("The file to which to redirect the standard output of the build"))
            .arg(Arg::new("stderr").long("stderr").action(ArgAction::Set).help("The file to which to redirect the error output of the build"))
        },
    }
}

pub fn handle_cmd(parsed_args: &ArgMatches) -> Result<(), String> {
    let derivation_path = parsed_args
        .get_one::<String>("DERIVATION")
        .ok_or("You must specify a derivation.")?;
    let build_dir = parsed_args
        .get_one::<String>("build-dir")
        .map_or_else(create_build_dir, |str| Ok(PathBuf::from(str)))?;
    let stdout_file = parsed_args
        .get_one::<String>("stdout")
        .map(File::create)
        .transpose()
        .map_err(|err| format!("Could not create the stdout file. Error: {}", err))?;
    let stderr_file = parsed_args
        .get_one::<String>("stderr")
        .map(File::create)
        .transpose()
        .map_err(|err| format!("Could not create the stderr file. Error: {}", err))?;
    let derivation = derivations::load_derivation(derivation_path)?;
    let nix_delegation_store = NixDelegationStore::default();
    let mut build_config = BuildConfig::new(&derivation, &build_dir, &nix_delegation_store);
    if let Some(stdout_file) = stdout_file.as_ref() {
        build_config.stdout_to_file(stdout_file);
    }
    if let Some(stderr_file) = stderr_file.as_ref() {
        build_config.stderr_to_file(stderr_file);
    }
    let result_code = build_derivation_sandboxed(&build_config)?;
    println!("{}", build_dir.to_str().unwrap());
    std::process::exit(result_code);
}

fn create_build_dir() -> Result<PathBuf, String> {
    tempdir()
        .map_err(|err| format!("Could not create the build directory. Error: {}", err))
        .map(|tmp_dir| tmp_dir.into_path())
}
