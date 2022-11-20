use std::collections::HashMap;

use crate::cmd::{to_cmd_err, RixSubCommand};
use crate::eval::{eval_str, Value};
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
    print_value(&eval_str(expr));
    println!();
    Ok(())
}

fn print_value(value: &Value) {
    match value {
        Value::AttrSet(hash_map) => print_attrset(&hash_map),
        Value::Bool(boolean) => print!("{}", boolean),
        Value::Float(float) => print!("{:.6}", float),
        Value::Int(int) => print!("{}", int),
        Value::Str(string) => print!("\"{}\"", string),
    }
}

fn print_attrset(hash_map: &HashMap<String, Value>) {
    print!("{{ ");
    for (attr_name, value) in hash_map {
        print!("{} = ", attr_name);
        print_value(value);
        print!("; ");
    }
    print!("}}");
}
