use clap::{App, Arg, SubCommand};

mod imgui_wrapper;
mod lua;
mod new;
mod run;
mod signal;

use crate::new::new;
use crate::run::run;

const VERSION: &str = "0.1.2";

fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    let matches = App::new("maple")
        .version(VERSION)
        .author("nalleyer")
        .about("make prototype easy")
        .subcommand(
            SubCommand::with_name("run")
                .about("run you lua folder or file")
                .version(VERSION)
                .author("nalleyer")
                .arg(Arg::with_name("INPUT").required(true)),
        )
        .subcommand(
            SubCommand::with_name("new")
                .about("generate lua scaffold")
                .version(VERSION)
                .author("nalleyer")
                .arg(Arg::with_name("FILENAME").required(true)),
        )
        .get_matches();

    if let Some(matches) = matches.subcommand_matches("run") {
        let input_path = matches.value_of("INPUT").unwrap();
        run(&input_path)?;
    }

    if let Some(matches) = matches.subcommand_matches("new") {
        let file_name = matches.value_of("FILENAME").unwrap();
        new(&file_name)?;
    }

    Ok(())
}
