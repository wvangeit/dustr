use clap::{CommandFactory, FromArgMatches};
use std::process;

mod core;

fn main() {
    // Parse using the shared Cli struct but display as "dustr-cli"
    let matches = core::Cli::command().name("dustr-cli").get_matches();
    let cli = core::Cli::from_arg_matches(&matches).unwrap_or_else(|e| e.exit());

    match core::print_disk_usage(
        &cli.dirname,
        cli.inodes,
        cli.nogrouping,
        cli.no_f,
        cli.json,
        cli.cross_mounts,
        cli.verbose,
        cli.live,
    ) {
        Ok(()) => {}
        Err(core::DustrError::Cancelled) => {
            // Clean exit on Ctrl-C
            process::exit(130);
        }
        Err(e) => {
            eprintln!("dustr-cli: {}", e);
            process::exit(1);
        }
    }
}
