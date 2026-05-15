use std::{env, process};

use grin::{Config, run};

fn main() {
    let args: Vec<String> = env::args().collect();
    let config = Config::build(&args).unwrap_or_else(|err| {
        eprintln!("{}", err);
        eprintln!(
            "Try `{} help` for a list of commands.",
            grin::program_invocation()
        );
        process::exit(1);
    });
    run(config);
}
