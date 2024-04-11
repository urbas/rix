pub mod eval;
pub mod transpile;
use clap::{ArgMatches, Command};
use colored::*;
use std::process::ExitCode;

use crate::eval::error::NixError;

pub struct RixSubCommand {
    pub name: &'static str,
    pub cmd: fn(Command) -> Command,
    pub handler: fn(&ArgMatches) -> Result<(), ExitCode>,
}

pub fn print_err(msg: NixError) {
    eprintln!("{}: {}", "error".red(), msg);
}

pub fn to_cmd_err(result: Result<(), NixError>) -> Result<(), ExitCode> {
    result.map_err(print_and_err)
}

pub fn print_and_err(msg: NixError) -> ExitCode {
    print_err(msg);
    ExitCode::FAILURE
}
