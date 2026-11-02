use clap::{Parser, Subcommand};
use std::path::PathBuf;

/// A CLI tool to create "warpzones" around your command line filesystem.
#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    /// Enables verbose logging.
    #[arg(short, long)]
    verbose: bool,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Adds a warpzone with the specified path.
    Add { name: String, path: PathBuf },
    /// Removes a warpzone with the name.
    Delete { name: String },
    /// Updates a warpzone's path with the name.
    Update { name: String, path: PathBuf },
    /// Lists all warpzones with their paths.
    List,
    /// Clears all warpzones.
    Clear,
    /// Initializes your shell configuration to accept warp as a command to actually warp.
    Init,
}

fn main() {
    let cli = Cli::parse();

    if cli.verbose {
        println!("You enabled verbose logging!")
    }

    // TODO: remove this when done
    #[allow(unused_variables)]
    match cli.command {
        Some(Commands::Add { name, path }) => add_warpzone(name, path),
        Some(Commands::Delete { name }) => {}
        Some(Commands::Update { name, path }) => {}
        Some(Commands::List) => {}
        Some(Commands::Clear) => {}
        Some(Commands::Init) => {}
        None => {}
    }
}

fn add_warpzone(name: String, path: PathBuf) {
    println!("Added {} warpzone: {}", name, path.display())
}
