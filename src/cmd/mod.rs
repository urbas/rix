pub mod build_derivation;
pub mod hash;
pub mod show_derivation;
use clap::{App, ArgMatches};

type CmdResult = Result<(), String>;
type CmdHandler = fn(&ArgMatches) -> CmdResult;

pub struct RixSubCommand<'a> {
    pub name: &'a str,
    pub cmd: fn(App<'a, 'a>) -> App<'a, 'a>,
    pub handler: &'a CmdHandler,
}
