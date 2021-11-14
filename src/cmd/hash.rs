use crate::hashes;
use clap::{App, Arg, ArgMatches, SubCommand};

pub const CMD_NAME: &str = "hash";

pub fn cmd<'a>() -> App<'a, 'a> {
    return SubCommand::with_name(CMD_NAME)
        .about("compute and convert cryptographic hashes")
        .subcommand(to_base_cmd("to-base16").about("convert hashes to base-16 representation"))
        .subcommand(
            to_base_cmd("to-base32").about("convert hashes to the Nix base-32 representation"),
        )
        .subcommand(
            to_base_cmd("to-base64").about("convert hashes to the Nix base-32 representation"),
        );
}

pub fn handle_cmd(parsed_args: &ArgMatches) -> Result<(), String> {
    if let Some(to_base_args) = parsed_args.subcommand_matches("to-base16") {
        to_base(to_base_args, hashes::to_base16)
    } else if let Some(to_base_args) = parsed_args.subcommand_matches("to-base32") {
        to_base(to_base_args, hashes::to_base32)
    } else if let Some(to_base_args) = parsed_args.subcommand_matches("to-base64") {
        to_base(to_base_args, hashes::to_base64)
    } else {
        Err("operation not supported".to_owned())
    }
}

fn to_base_cmd(name: &str) -> App {
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

fn to_base(
    to_base_args: &clap::ArgMatches,
    to_base_fn: impl Fn(&[u8], &mut String),
) -> Result<(), String> {
    let mut hash_strs = to_base_args
        .values_of("HASHES")
        .ok_or("Please specify some hashes.")?;
    let hash_type = to_base_args
        .value_of("type")
        .ok_or("SRI hashes not supported yet.")?;
    match hash_type {
        "md5" => hash_type_to_base::<_, 16>(&mut hash_strs, to_base_fn),
        "sha1" => hash_type_to_base::<_, 20>(&mut hash_strs, to_base_fn),
        "sha256" => hash_type_to_base::<_, 32>(&mut hash_strs, to_base_fn),
        "sha512" => hash_type_to_base::<_, 64>(&mut hash_strs, to_base_fn),
        _ => Err("hash type not supported".to_owned()),
    }
}

fn hash_type_to_base<F: Fn(&[u8], &mut String), const N: usize>(
    hash_strs: &mut clap::Values,
    to_base_fn: F,
) -> Result<(), String> {
    let mut out_string = String::new();
    let mut hash_buf = [0; N];
    hash_strs.try_for_each(|hash_str| {
        hashes::parse::<N>(hash_str, &mut hash_buf).map(|_| {
            out_string.clear();
            to_base_fn(&hash_buf, &mut out_string);
            println!("{}", out_string);
        })
    })
}
