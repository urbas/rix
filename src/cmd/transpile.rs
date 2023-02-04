use crate::cmd::{to_cmd_err, RixSubCommand};
use crate::eval::nix_v8;
use clap::{Arg, ArgAction, ArgMatches};

pub fn cmd() -> RixSubCommand {
    return RixSubCommand {
        name: "transpile",
        handler: |args| to_cmd_err(handle_cmd(args)),
        cmd: |subcommand| {
            subcommand
                .about("transpiles the given nix expression file into JavaScript.")
                .arg(Arg::new("EXPRESSION").help("The nix expression to transpile."))
                .arg(
                    Arg::new("expr")
                        .long("expr")
                        .action(ArgAction::SetTrue)
                        .help("The 'EXPRESSION' argument will be treated as an expression rather than a file."),
                )
        },
    };
}

pub fn handle_cmd(parsed_args: &ArgMatches) -> Result<(), String> {
    let expression = parsed_args
        .get_one::<String>("EXPRESSION")
        .ok_or("You must provide a single expression to transpile.")?;
    let is_expression = parsed_args.get_one::<bool>("expr").unwrap();
    if *is_expression {
        let js_source = nix_v8::emit_module(&expression)
            .map_err(|_err| "Failed to transpile the expression.".to_owned())?;
        print!("{js_source}");
    } else {
        todo!("Support to transpile files is not yet implemented.")
    }
    Ok(())
}
