use clap::{App, Arg, SubCommand};

mod imgui_wrapper;
mod lua;
mod run;

use crate::run::run;

fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    let matches = App::new("maple")
        .version("0.1.0")
        .author("nalleyer")
        .about("make prototype easy")
        .subcommand(
            SubCommand::with_name("run")
                .about("run you lua folder or file")
                .version("0.1.0")
                .author("nalleyer")
                .arg(Arg::with_name("INPUT").required(true)),
        )
        .get_matches();

    if let Some(matches) = matches.subcommand_matches("run") {
        let input_path = matches.value_of("INPUT").unwrap();
        run(&input_path)?;
    }

    Ok(())
}
