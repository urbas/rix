pub mod build_derivation;
pub mod hash;
pub mod show_derivation;
use clap::{ArgMatches, Command};
use colored::*;
use std::process::ExitCode;

type CmdResult = Result<(), ExitCode>;
type CmdHandler = fn(&ArgMatches) -> CmdResult;

pub struct RixSubCommand<'a> {
    pub name: &'a str,
    pub cmd: fn(Command<'a>) -> Command<'a>,
    pub handler: &'a CmdHandler,
}

pub fn print_err(msg: &str) {
    eprintln!("{}: {}", "error".red(), msg);
}

pub fn to_cmd_err<T>(result: Result<T, String>) -> Result<T, ExitCode> {
    result.or_else(|err| {
        print_err(&err);
        Err(ExitCode::FAILURE)
    })
}

pub fn print_and_err(msg: &str) -> ExitCode {
    print_err(msg);
    ExitCode::FAILURE
}
