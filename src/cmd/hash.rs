use crate::cmd::{CmdHandler, CmdResult, RixSubCommand};
use crate::hashes;
use clap::{Arg, ArgMatches, Command, SubCommand};

pub fn cmd<'a>() -> RixSubCommand<'a> {
    return RixSubCommand {
        name: "hash",
        handler: &(handle_cmd as CmdHandler),
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
    };
}

pub fn handle_cmd(parent_args: &ArgMatches) -> CmdResult {
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

fn to_base_cmd(name: &str) -> Command {
    SubCommand::with_name(name)
        .arg(
            Arg::with_name("HASHES")
                .multiple(true)
                .help("A list of hashes to convert."),
        )
        .arg(
            Arg::with_name("type")
                .long("type")
                .value_name("hash-algo")
                .possible_values(&["md5", "sha1", "sha256", "sha512"])
                .help("Hash algorithm of input HASHES. Optional as can also be extracted from SRI hash itself.")
                .takes_value(true),
        )
}

fn handle_to_base_cmd<F>(args: &clap::ArgMatches, to_base_fn: F) -> CmdResult
where
    F: Fn(&hashes::Hash) -> String,
{
    let mut hash_strs = args
        .values_of("HASHES")
        .ok_or("Please specify some hashes.")?;
    let type_arg = args.value_of("type").unwrap_or("sri");

    if let Some(hash_type) = hashes::HashType::from_str(type_arg) {
        return hash_strs.try_for_each(|hash_str| print_hash(hash_str, hash_type, &to_base_fn));
    } else if type_arg == "sri" {
        return sri_to_base(&mut hash_strs, &to_base_fn);
    }
    return Err("hash type not supported".to_owned());
}

fn sri_to_base<F>(hash_strs: &mut clap::Values, to_base_fn: F) -> CmdResult
where
    F: Fn(&hashes::Hash) -> String,
{
    hash_strs.try_for_each(|hash_str| {
        let (hash_type, hash_str) = hashes::sri_hash_components(hash_str)?;
        if let Some(hash_type) = hashes::HashType::from_str(hash_type) {
            return print_hash(hash_str, hash_type, &to_base_fn);
        }
        return Err(format!("Hash type '{}' not supported.", hash_type));
    })
}

fn print_hash<F>(hash_str: &str, hash_type: hashes::HashType, to_base_fn: F) -> CmdResult
where
    F: Fn(&hashes::Hash) -> String,
{
    hashes::parse(hash_str, hash_type).map(|hash| println!("{}", to_base_fn(&hash)))
}
