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
    match eval_str(expr) {
        Value::Bool(boolean) => println!("{}", boolean),
        Value::Int(int) => println!("{}", int),
        Value::Float(float) => println!("{:.6}", float),
    }
    Ok(())
}
