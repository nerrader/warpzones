use clap::{Parser, Subcommand};
use std::path::PathBuf;

/// Simple program to greet a person
#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    /// Turns on verbose logging.
    #[arg(short, long)]
    verbose: bool,
}

#[derive(Subcommand)]
enum Commands {
    Add { name: String, path: PathBuf },
    Delete { name: String },
    Update { name: String, path: PathBuf },
    List {},
    Clear {},
}

fn main() {
    let cli = Cli::parse();

    if cli.verbose {
        println!("You enabled verbose logging!")
    }
}
