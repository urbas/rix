use clap::{App, SubCommand};

pub fn cmd_show_derivations() -> App<'static, 'static> {
    return SubCommand::with_name("show-derivation")
        .about("show the contents of a store derivation");
}
