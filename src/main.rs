use clap::Parser;
use std::process;

mod core;

/// Command-line arguments structure
#[derive(Parser)]
#[command(name = "dustr-cli")]
#[command(about = "Show disk usage statistics", long_about = None)]
struct Cli {
    /// Directory to analyze
    #[arg(default_value = ".")]
    dirname: String,

    /// Count inodes instead of disk usage
    #[arg(short, long)]
    inodes: bool,

    /// Don't use thousand separators
    #[arg(short = 'g', long)]
    nogrouping: bool,

    /// Don't append file type indicators
    #[arg(short = 'f', long = "noF")]
    no_f: bool,

    /// Output results as JSON
    #[arg(short, long)]
    json: bool,

    /// Cross mount boundaries (by default stays on the same filesystem)
    #[arg(short = 'x', long)]
    cross_mounts: bool,

    /// Show directories being traversed
    #[arg(short, long)]
    verbose: bool,

    /// Live-update statistics table during traversal
    #[arg(short, long)]
    live: bool,
}

fn main() {
    let cli = Cli::parse();

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
            eprintln!("dustr: {}", e);
            process::exit(1);
        }
    }
}
