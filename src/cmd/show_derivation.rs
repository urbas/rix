use crate::derivations::Derivation;
use crate::parsers::derivations::parse_derivation;
use clap::{App, Arg, ArgMatches, SubCommand};
use serde::ser::{SerializeMap, Serializer};
use serde_json;
use std::fs;

pub const CMD_NAME: &str = "show-derivation";

pub fn cmd() -> App<'static, 'static> {
    return SubCommand::with_name(CMD_NAME)
        .about("show the contents of a store derivation")
        .arg(Arg::with_name("INSTALLABLES").multiple(true).help(
            "A list of derivation files. Other types of installables are not yet supported.",
        ));
}

pub fn handle_cmd(parsed_args: &ArgMatches) -> Result<(), String> {
    let mut installables = parsed_args
        .values_of("INSTALLABLES")
        .ok_or("Please specify some derivation files.")?;
    show_derivations(&mut installables)
}

fn show_derivations<'a>(drv_paths: &mut impl Iterator<Item = &'a str>) -> Result<(), String> {
    let mut json_serializer = serde_json::Serializer::new(std::io::stdout());
    let mut map_serializer = json_serializer
        .serialize_map(None)
        .map_err(|_| "Failed to initialize JSON serialization.")?;

    let error_maybe =
        drv_paths.try_for_each(|drv_path| show_derivation(&mut map_serializer, drv_path));

    // this makes sure we produce valid JSON even if there's a failure while dumping the derivations above
    map_serializer.end().unwrap();

    error_maybe
}

fn show_derivation(serializer: &mut impl SerializeMap, drv_path: &str) -> Result<(), String> {
    serializer
        .serialize_entry(drv_path, &to_derivation(drv_path)?)
        .map_err(|_| format!("Failed to serialize derivation '{}' to JSON.", drv_path))
}

fn to_derivation(drv_path: &str) -> Result<Derivation, String> {
    let content = fs::read_to_string(drv_path)
        .map_err(|err| format!("Failed to read '{}': {}", drv_path, err))?;
    parse_derivation(&content)
        .map(|(_, derivation)| derivation)
        .map_err(|err| format!("Failed to parse '{}': {}", drv_path, err))
}
