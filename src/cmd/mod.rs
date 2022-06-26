pub mod build_derivation;
pub mod hash;
pub mod show_derivation;
use clap::{ArgMatches, Command};

type CmdResult = Result<(), String>;
type CmdHandler = fn(&ArgMatches) -> CmdResult;

pub struct RixSubCommand<'a> {
    pub name: &'a str,
    pub cmd: fn(Command<'a>) -> Command<'a>,
    pub handler: &'a CmdHandler,
}
