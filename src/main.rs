use std::{
    collections::HashMap,
    fs,
    path::{self, Path, PathBuf},
};

use clap::{Parser, Subcommand};
use colored::Colorize;
use directories::ProjectDirs;

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

type Warpzones = HashMap<String, PathBuf>;
fn main() {
    let cli = Cli::parse();

    let data_dir = ProjectDirs::from("com", "nerrader", "warpzones")
        .expect("could not determine project directories")
        .data_local_dir()
        .to_path_buf();
    fs::create_dir_all(&data_dir).expect("could not create project directories");
    let warpzone_path = data_dir.join("warpzones.json");
    if cli.verbose {
        println!("You enabled verbose logging!")
    }

    let mut warpzones: Warpzones = load_warpzones(&warpzone_path).unwrap_or_default();

    // TODO: remove this when done
    #[allow(unused_variables)]
    match cli.command {
        Some(Commands::Add { name, path }) => add_warpzone(name, path, &mut warpzones),
        Some(Commands::Delete { name }) => delete_warpzone(name, &mut warpzones),
        Some(Commands::Update { name, path }) => {
            update_warpzone(name, path, &mut warpzones);
        }
        Some(Commands::List) => list_warpzones(&warpzones),
        Some(Commands::Clear) => warpzones.clear(),
        Some(Commands::Init) => {}
        None => {}
    }
    save_warpzones(&warpzone_path, warpzones);
}

fn add_warpzone(name: String, path: PathBuf, warpzones: &mut Warpzones) {
    let absolute_path =
        path::absolute(&path).unwrap_or_else(|_| panic!("invalid path: {}", path.display()));

    if warpzones.contains_key(&name) {
        println!(
            "{} {}",
            "warning: there is already a warpzone named: ".yellow(),
            name.yellow()
        );
        std::process::exit(0);
    }
    warpzones.insert(name.clone(), absolute_path.clone());
    println!("added {} warpzone: {}", name, absolute_path.display());
}

fn update_warpzone(name: String, path: PathBuf, warpzones: &mut Warpzones) {
    let absolute_path =
        path::absolute(&path).unwrap_or_else(|_| panic!("invalid path: {}", path.display()));

    if !warpzones.contains_key(&name) {
        println!(
            "{} {}",
            "warning: there is no warpzone named: ".yellow(),
            name.yellow()
        );
        std::process::exit(0);
    }
    // basically overwrites the old value
    warpzones.insert(name.clone(), absolute_path.clone());
    println!("updated {} warpzone: {}", name, absolute_path.display());
}

fn delete_warpzone(name: String, warpzones: &mut Warpzones) {
    match warpzones.remove(&name) {
        Some(_) => println!("deleted {} warpzone", name),
        None => println!("key not found in warpzones: {}", name),
    }
}

fn save_warpzones(path: &Path, warpzones: Warpzones) {
    let json =
        serde_json::to_string_pretty(&warpzones).expect("could not serialize warpzones to json");
    fs::write(path, json)
        .unwrap_or_else(|_| panic!("could not write json to savefile: {}", path.display()));
}

fn load_warpzones(path: &Path) -> Result<Warpzones, Box<dyn std::error::Error>> {
    let contents = fs::read_to_string(path)?;
    let warpzones = serde_json::from_str(&contents)?;
    Ok(warpzones)
}

fn list_warpzones(warpzones: &Warpzones) {
    for (name, path) in warpzones {
        println!("{} => {}", name, path.display())
    }
}
