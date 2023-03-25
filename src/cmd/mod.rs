pub mod build_derivation;
pub mod eval;
pub mod hash;
pub mod show_derivation;
pub mod transpile;
use clap::{ArgMatches, Command};
use colored::*;
use std::process::ExitCode;

pub struct RixSubCommand {
    pub name: &'static str,
    pub cmd: fn(Command) -> Command,
    pub handler: fn(&ArgMatches) -> Result<(), ExitCode>,
}

pub fn print_err(msg: &str) {
    eprintln!("{}: {}", "error".red(), msg);
}

pub fn to_cmd_err(result: Result<(), String>) -> Result<(), ExitCode> {
    result.map_err(|err| print_and_err(&err))
}

pub fn print_and_err(msg: &str) -> ExitCode {
    print_err(msg);
    ExitCode::FAILURE
}
