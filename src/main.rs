use std::{
    collections::HashMap,
    env::current_exe,
    fs::{self, OpenOptions},
    io::Write,
    path::{self, Path, PathBuf},
    process::Command,
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
    List {
        #[arg(short, long, help = "Gets the path of a specific warpzone")]
        warpzone: Option<String>,
    },
    /// Clears all warpzones.
    Clear,
    /// Initializes your shell configuration to accept warp as a command.
    /// This is required to actually warp.
    Init {
        #[arg(help = "Shell to configure. Supported: ps (powershell)")]
        shell: String,
    },
}

enum JsonLoadError {
    Io(std::io::Error),
    Json(serde_json::Error),
}

impl From<std::io::Error> for JsonLoadError {
    fn from(error: std::io::Error) -> Self {
        JsonLoadError::Io(error)
    }
}

impl From<serde_json::Error> for JsonLoadError {
    fn from(error: serde_json::Error) -> Self {
        JsonLoadError::Json(error)
    }
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
        println!("you enabled verbose logging!")
    }

    let result = load_warpzones(&warpzone_path);
    let mut warpzones: Warpzones = match result {
        Ok(warpzones) => warpzones,
        Err(JsonLoadError::Io(error)) => {
            eprintln!(
                "{}",
                "failed to read json file, defaulting to empty warpzones.".red()
            );
            eprintln!("{}", format!("detailed message: {}", error).red());
            Warpzones::new()
        }
        Err(JsonLoadError::Json(error)) => {
            eprintln!(
                "{}",
                "failed to parse json, data might be corrupted, defaulting to empty warpzones."
                    .red()
            );
            eprintln!("{}", format!("detailed message: {}", error).red());
            Warpzones::new()
        }
    };

    match cli.command {
        Some(Commands::Add { name, path }) => add_warpzone(name, path, &mut warpzones),
        Some(Commands::Delete { name }) => delete_warpzone(name, &mut warpzones),
        Some(Commands::Update { name, path }) => {
            update_warpzone(name, path, &mut warpzones);
        }
        Some(Commands::List { warpzone }) => list_warpzones(&warpzones, warpzone),
        Some(Commands::Clear) => {
            warpzones.clear();
            println!("{}", "cleared all warpzones!".green())
        }
        Some(Commands::Init { shell }) => init_shell(shell),
        None => {}
    }
    save_warpzones(&warpzone_path, &warpzones);
}

fn add_warpzone(name: String, path: PathBuf, warpzones: &mut Warpzones) {
    // Adds a warpzone by its name and path to the mutable warpzones hashmap.
    let absolute_path =
        path::absolute(&path).unwrap_or_else(|_| panic!("invalid path: {}", path.display()));

    if warpzones.contains_key(&name) {
        eprintln!(
            "{} {}",
            "user-error: there is already a warpzone named ".red(),
            name.red()
        );
        return;
    }
    warpzones.insert(name.clone(), absolute_path.clone());
    println!(
        "{}",
        format!("added {} warpzone: {}", name, absolute_path.display()).green()
    );
}

fn update_warpzone(name: String, path: PathBuf, warpzones: &mut Warpzones) {
    // Updates a warpzone path by its name
    let absolute_path =
        path::absolute(&path).unwrap_or_else(|_| panic!("invalid path: {}", path.display()));

    if !warpzones.contains_key(&name) {
        eprintln!(
            "{} {}",
            "user-error: there is no warpzone named: ".red(),
            name.yellow()
        );
        return;
    }
    // basically overwrites the old value
    warpzones.insert(name.clone(), absolute_path.clone());
    println!(
        "{}",
        format!("updated {} warpzone: {}", name, absolute_path.display()).green()
    );
}

fn delete_warpzone(name: String, warpzones: &mut Warpzones) {
    // Deletes a warpzone by its name.
    match warpzones.remove(&name) {
        Some(_) => println!(
            "{}",
            format!("successfully deleted {} warpzone", name).green()
        ),
        None => eprintln!(
            "{}",
            format!("user-error: {} not found in warpzones", name).red()
        ),
    }
}

fn save_warpzones(path: &Path, warpzones: &Warpzones) {
    // Writes the warpzones hashmap to a JSON file.
    let json =
        serde_json::to_string_pretty(&warpzones).expect("could not serialize warpzones to json");
    fs::write(path, json)
        .unwrap_or_else(|_| panic!("could not write json to savefile: {}", path.display()));
}

fn load_warpzones(path: &Path) -> Result<Warpzones, JsonLoadError> {
    // Loads the warpzones hashmap from a JSON file.
    let contents = fs::read_to_string(path)?;
    let warpzones = serde_json::from_str(&contents)?;
    Ok(warpzones)
}

fn list_warpzones(warpzones: &Warpzones, specific_warpzone: Option<String>) {
    // Lists the warpzones in (name => path) format.
    // If specific_warpzone is a real name in warpzones, only prints the path.
    // Though that option should only be used for the the warp in init.
    if let Some(warpzone_name) = specific_warpzone {
        match warpzones.get(&warpzone_name) {
            Some(path) => println!("{}", path.display()),
            None => eprintln!(
                "{}",
                format!("user-error: warpzone {} does not exist", warpzone_name).red()
            ),
        }
        return;
    }
    for (name, path) in warpzones {
        println!("{} => {}", name, path.display())
    }
}

fn init_shell(shell: String) {
    // Matches the shell to a specific init function.
    match shell.as_str() {
        "powershell" => init_powershell(),
        _ => eprintln!("{}", format!("user-error: {} not supported!", shell).red()),
    }
}

// TODO: Refactor this function to make it more readable
fn init_powershell() {
    // Initializes the 'warp' command for powershell and ps7
    let powershell_output = Command::new("powershell")
        .args(["-NoProfile", "-Command", "$PROFILE"])
        .output()
        .unwrap();
    let profile = String::from_utf8_lossy(&powershell_output.stdout)
        .trim()
        .to_string();
    let mut file = OpenOptions::new()
        .append(true)
        .create(true)
        .open(&profile)
        .expect("could not open powershell profile for some reason");

    let mut profile_contents = fs::read_to_string(&profile).unwrap();

    let exe_path = current_exe().expect("could not find executable path");

    let start_marker = "# WARPZONE BLOCK START";
    let end_marker = "# WARPZONE BLOCK END";
    let warp_function = format!(
        r##"{}
function warp ($warpzone) {{

    $path = & "{}" list --warpzone $warpzone

    if ($LASTEXITCODE -eq 0 -and $path) {{
        Set-Location $path.Trim()
    }}
}}
{}"##,
        start_marker,
        exe_path.display(),
        end_marker
    );

    let start_marker_index = profile_contents.find(start_marker);
    let end_marker_index = profile_contents.find(end_marker);

    if let (Some(start), Some(end_offset)) = (start_marker_index, end_marker_index) {
        let end = end_offset + end_marker.len();
        profile_contents.replace_range(start..end, &warp_function);
        fs::write(&profile, &profile_contents).expect("could not write to powershell profile");
        return;
    }

    // added a newline at the start cause it looks better with it
    // not in the string itself cause it would interfere with the replacing
    writeln!(file, "\n\n{}", warp_function).expect("could not write to powershell profile");
}
