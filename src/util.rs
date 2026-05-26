//! Various helper functions and methods

use std::{collections::HashMap, path::PathBuf, sync::Arc};

use tokio::{fs, io, process};

use crate::{game::Game, settings::Settings};

pub mod slug;

/// Run a shell command
pub async fn run(command: &str) -> tokio::io::Result<()> {
    process::Command::new("sh")
        .args(["-c", command])
        .status()
        .await?;

    Ok(())
}

/// Creates all required directories for the app if they don't exist
pub async fn ensure_directories_exist() {
    fs::create_dir_all(downloads()).await.unwrap();
    fs::create_dir_all(prefix()).await.unwrap();
}

/* Useful paths */
/// The base data directory of the app (`~/.local/share/freebie`)
pub fn base() -> PathBuf {
    dirs::data_dir().unwrap().join("freebie2") // TODO: Change from `freebie2` to `freebie`
}

/// The directory that temporarily contains downloaded torrents (`~/.local/share/freebie/downloads`)
pub fn downloads() -> PathBuf {
    base().join("downloads")
}

/// The wine prefix directory (`~/.local/share/freebie/prefix`)
pub fn prefix() -> PathBuf {
    base().join("prefix")
}

/// The user's desktop directory inside of the wine prefix (`~/.local/share/freebie/prefix/drive_c/users/Public/Desktop`)
pub fn wine_desktop() -> PathBuf {
    prefix()
        .join("drive_c")
        .join("users")
        .join("Public")
        .join("Desktop")
}

/// The users `Games` directory inside of the wine prefix (`~/.local/share/freebie/prefix/drive_c/Games`)
pub fn wine_games() -> PathBuf {
    prefix().join("drive_c").join("Games")
}

/// Get the path of the `.json` file that saves the installed games data
pub fn installed_games_file() -> PathBuf {
    base().join("installed_games.json")
}

/// Get the path of the `.json` file that contains the users settings
pub fn settings_file() -> PathBuf {
    base().join("settings.json")
}

/// Retrieve the users settings from disk
pub async fn settings() -> anyhow::Result<Settings> {
    let string = fs::read_to_string(settings_file()).await?;
    Ok(serde_json::from_str(&string)?)
}

/// Get all of the installed games from the `installed_games_file()` path. or an empty `HashMap`
/// if the file doesn't exist yet or failed to be read.
pub async fn installed_games() -> anyhow::Result<HashMap<String, Arc<Game>>> {
    let Ok(string) = tokio::fs::read_to_string(installed_games_file()).await else {
        return Ok(HashMap::new());
    };

    Ok(serde_json::from_str(&string)?)
}

/* Wine utils */
/// Run an `umu-launcher` command using the correct wine prefix and proton versions.
pub async fn umu(args: &[&str]) -> io::Result<()> {
    println!("Running umu command: umu-run {}", args.join(" "));
    process::Command::new("umu-run")
        .args(args)
        .env("WINEPREFIX", prefix().into_os_string()) // Set correct wine prefix
        .env("PROTONPATH", "GE-Proton") // Download and use the latest proton-ge
        .status()
        .await?;

    Ok(())
}

/// Mute and unmute the audio from the wine prefix.
/// This is useful because the repack exe plays music that can't be turned off via command line arguments.
pub async fn set_prefix_mute(muted: bool) -> io::Result<()> {
    let driver = if muted { "disabled" } else { "pulse" };
    umu(&["winetricks", &format!("sound={driver}")]).await?;
    Ok(())
}
