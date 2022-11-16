pub mod build_derivation;
pub mod eval;
pub mod hash;
pub mod show_derivation;
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
    result.or_else(|err| {
        print_err(&err);
        Err(ExitCode::FAILURE)
    })
}

pub fn print_and_err(msg: &str) -> ExitCode {
    print_err(msg);
    ExitCode::FAILURE
}
