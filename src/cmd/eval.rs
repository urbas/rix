use std::collections::HashMap;

use crate::cmd::{to_cmd_err, RixSubCommand};
use crate::eval::nix_v8;
use crate::eval::types::Value;
use clap::{Arg, ArgAction, ArgMatches};

pub fn cmd() -> RixSubCommand {
    return RixSubCommand {
        name: "eval",
        handler: |args| to_cmd_err(handle_cmd(args)),
        cmd: |subcommand| {
            subcommand
                .about("evaluates the given expression and prints the result")
                .arg(Arg::new("INSTALLABLE").help("The thing to evaluate."))
                .arg(
                    Arg::new("expr")
                        .long("expr")
                        .action(ArgAction::Set)
                        .help("The expression to evaluate. Installables are treated as attribute paths of the attrset returned by the expression."),
                )
        },
    };
}

pub fn handle_cmd(parsed_args: &ArgMatches) -> Result<(), String> {
    let expr = parsed_args
        .get_one::<String>("expr")
        .ok_or("You must use the '--expr' option. Nothing else is implemented :)")?;
    print_value(&nix_v8::evaluate(expr)?);
    println!();
    Ok(())
}

fn print_value(value: &Value) {
    match value {
        Value::AttrSet(hash_map) => print_attrset(&hash_map),
        Value::Bool(boolean) => print!("{boolean}"),
        Value::Float(float) => print!("{float}"),
        Value::Int(int) => print!("{int}"),
        Value::Lambda => print!("<LAMBDA>"),
        Value::List(vector) => print_list(vector),
        Value::Str(string) => print!("\"{string}\""),
    }
}

fn print_list(vector: &Vec<Value>) {
    print!("[ ");
    for value in vector {
        print_value(value);
        print!(" ");
    }
    print!("]");
}

fn print_attrset(hash_map: &HashMap<String, Value>) {
    print!("{{ ");
    for (attr_name, value) in hash_map {
        print!("{attr_name} = ");
        print_value(value);
        print!("; ");
    }
    print!("}}");
}
