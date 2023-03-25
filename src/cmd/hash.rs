use crate::cmd::{to_cmd_err, RixSubCommand};
use crate::hashes;
use clap::{Arg, ArgAction, ArgMatches, Command};

pub fn cmd() -> RixSubCommand {
    RixSubCommand {
        name: "hash",
        handler: |args| to_cmd_err(handle_cmd(args)),
        cmd: |subcommand| {
            subcommand
                .about("compute and convert cryptographic hashes")
                .subcommand(
                    to_base_cmd("to-base16").about("convert hashes to base-16 representation"),
                )
                .subcommand(
                    to_base_cmd("to-base32")
                        .about("convert hashes to the Nix base-32 representation"),
                )
                .subcommand(
                    to_base_cmd("to-base64").about("convert hashes to base-64 representation"),
                )
                .subcommand(
                    to_base_cmd("to-sri").about("convert hashes to SRI base-64 representation"),
                )
        },
    }
}

pub fn handle_cmd(parent_args: &ArgMatches) -> Result<(), String> {
    if let Some(args) = parent_args.subcommand_matches("to-base16") {
        handle_to_base_cmd(args, hashes::to_base16)
    } else if let Some(args) = parent_args.subcommand_matches("to-base32") {
        handle_to_base_cmd(args, hashes::to_base32)
    } else if let Some(args) = parent_args.subcommand_matches("to-base64") {
        handle_to_base_cmd(args, hashes::to_base64)
    } else if let Some(args) = parent_args.subcommand_matches("to-sri") {
        handle_to_base_cmd(args, hashes::to_sri)
    } else {
        Err("operation not supported".to_owned())
    }
}

fn to_base_cmd(name: &'static str) -> Command {
    Command::new(name)
        .arg(
            Arg::new("HASHES")
                .action(ArgAction::Append)
                .help("A list of hashes to convert."),
        )
        .arg(
            Arg::new("type")
                .long("type")
                .value_name("hash-algo")
                .value_parser(["md5", "sha1", "sha256", "sha512"])
                .help("Hash algorithm of input HASHES. Optional as can also be extracted from SRI hash itself."),
        )
}

fn handle_to_base_cmd<F>(args: &clap::ArgMatches, to_base_fn: F) -> Result<(), String>
where
    F: Fn(&hashes::Hash) -> String,
{
    let mut hash_strs = args
        .get_many::<String>("HASHES")
        .ok_or("Please specify some hashes.")?;
    let type_arg = args
        .get_one::<String>("type")
        .map(|s| s.as_str())
        .unwrap_or("sri");

    if let Ok(hash_type) = type_arg.parse() {
        return hash_strs.try_for_each(|hash_str| print_hash(hash_str, hash_type, &to_base_fn));
    } else if type_arg == "sri" {
        return sri_to_base(hash_strs, &to_base_fn);
    }
    Err("hash type not supported".to_owned())
}

fn sri_to_base<'a, F>(
    mut hash_strs: impl Iterator<Item = &'a String>,
    to_base_fn: F,
) -> Result<(), String>
where
    F: Fn(&hashes::Hash) -> String,
{
    hash_strs.try_for_each(|hash_str| {
        let (hash_type, hash_str) = hashes::sri_hash_components(hash_str)?;
        let hash_type: hashes::HashType = hash_type.parse()?;
        print_hash(hash_str, hash_type, &to_base_fn)
    })
}

fn print_hash<F>(hash_str: &str, hash_type: hashes::HashType, to_base_fn: F) -> Result<(), String>
where
    F: Fn(&hashes::Hash) -> String,
{
    hashes::parse(hash_str, hash_type).map(|hash| println!("{}", to_base_fn(&hash)))
}
