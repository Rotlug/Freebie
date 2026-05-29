//! Search, download and install games from `fitgirl-repacks.site`

use librqbit::{AddTorrent, AddTorrentOptions};
use scraper::{Html, Selector};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    fs,
    io::Write,
    path::{Path, PathBuf},
    sync::{Arc, RwLock},
    time::{Duration, Instant},
};
use tokio::{self};

use crate::{
    error::{DownloadError, InstallError},
    igdb,
    util::{
        applications, desktop, downloads, icons, run, set_prefix_mute, slug::SlugExt, umu,
        wine_desktop, wine_games,
    },
};

/// A Video game that can be downloaded and installed.
#[derive(Debug, Serialize, Deserialize)]
pub struct Game {
    /// A url to the page that contains the game's magnet link
    pub link: String,
    /// The file size of the game after installation
    /// (for example: `"25.1 GB"`)
    pub size: String,
    /// The url-friendly version of the games title
    /// (for example: "`tetris-effect-connected`")
    pub slug: String,

    /// The games metadata from IGDB, or `None` if it hasn't been fetched yet.
    pub metadata: Option<igdb::Metadata>,

    /// The games current installation state.
    pub state: Arc<RwLock<State>>,
}

/// A Games current installation state
#[derive(Default, Debug, Serialize, Deserialize, Clone)]
pub enum State {
    /// The game is uninstalled
    #[default]
    Uninstalled,
    /// Steps in preperation for downloading (finding the download link, adding the torrent to the session, etc..)
    Preparing,
    /// The game is currently in the middle of being downloaded. the
    /// `String` contains a human-readable version of the current percentage.
    /// (for example: `"25.46%"`)
    Downloading(String),
    /// The game's installer is currently running.
    Installing,
    /// The game is already installed.
    Installed {
        /// The game's installation directory. usually this is the directory above
        /// The game's `exe` location.
        path: PathBuf,
        /// The location for the games `exe` or `lnk` file.
        exe: PathBuf,
        /// The games total time played
        time_played: Duration,
        /// Is the game currently open
        #[serde(skip)]
        is_open: bool,
    },
}

impl Game {
    /// Download a game using `link`. Returns the output directory of the download.
    pub async fn download(
        &self,
        session: &Arc<librqbit::Session>,
        terminal_output: bool,
    ) -> Result<PathBuf, DownloadError> {
        *self.state.write().unwrap() = State::Preparing;

        let magnet = {
            let html = reqwest::get(&self.link).await?.text().await?;
            let document = Html::parse_document(&html);

            // Find first magnet link in page
            document
                .select(&Selector::parse("a").unwrap())
                .find_map(|link| {
                    let href = link.attr("href")?;

                    if href.starts_with("magnet") {
                        // Crucial: return an owned String so it doesn't borrow from 'document'
                        return Some(href.to_string());
                    }

                    None
                })
                .ok_or(DownloadError::MagnetLinkNotFound)?
        };

        let output_dir = downloads().join(&self.slug);

        // Start downloading
        let torrent_handle = session
            .add_torrent(
                AddTorrent::from_url(magnet),
                Some(AddTorrentOptions {
                    output_folder: Some(output_dir.display().to_string()),
                    ..Default::default()
                }),
            )
            .await
            .map_err(|_| DownloadError::TorrentError("Failed to add torrent"))?
            .into_handle()
            .ok_or(DownloadError::TorrentError("Failed to get torrent handle"))?;

        torrent_handle.wait_until_initialized().await.map_err(|_| {
            DownloadError::TorrentError("Failed to wait until torrent is initialized")
        })?;

        // Start download
        let track_progress = async {
            loop {
                let progress = torrent_handle
                    .stats()
                    .progress_percent_human_readable()
                    .to_string();

                *self.state.write().unwrap() = State::Downloading(progress.clone());

                if terminal_output {
                    print!("\rDownloading progress: {progress}");
                    let _ = std::io::stdout().flush();
                }

                tokio::time::sleep(Duration::from_secs(1)).await;
            }
        };

        let wait_for_completion = torrent_handle.wait_until_completed();

        tokio::select! {
            _ = track_progress => {},
            _ = wait_for_completion => {}
        }

        session
            .delete(torrent_handle.id().into(), false)
            .await
            .map_err(|_| DownloadError::TorrentError("Failed to delete torrent"))?; // TODO: Configurable seeding time

        Ok(output_dir)
    }

    /// Given the `download_dir`, look for the `setup.exe` file and install the game to the correct location.
    /// Returns the path to the executable OR the path to the desktop shortcut (`.lnk` file)
    pub async fn install(&self, download_dir: impl AsRef<Path>) -> Result<PathBuf, InstallError> {
        // Look for the setup.exe file
        *self.state.write().unwrap() = State::Installing;
        let setup = download_dir.as_ref().join("setup.exe");
        if !fs::exists(&setup)? {
            return Err(InstallError::SetupExeNotFound);
        }

        // Run the installer
        set_prefix_mute(true).await?;
        umu(&[
            &setup.to_string_lossy(),
            "/VERYSILENT",
            &format!(r"/DIR=C:\Games\{}", self.slug),
        ])
        .await?;
        set_prefix_mute(false).await?;

        // Remove the download directory
        tokio::fs::remove_dir_all(download_dir).await?;

        // Look for a matching .desktop shortcut
        for entry in fs::read_dir(wine_desktop())?.flatten() {
            let Ok(file_name) = entry.file_name().into_string() else {
                continue;
            };
            let Some(file_name) = file_name.strip_suffix(".lnk") else {
                continue;
            };

            if self.slug.contains(&file_name.ultra_slug()) {
                *self.state.write().unwrap() = State::Installed {
                    path: wine_games().join(&self.slug),
                    exe: entry.path(),
                    time_played: Duration::default(),
                    is_open: false,
                };

                return Ok(entry.path());
            }
        }

        // No Desktop shortcut has been found, installation failed.
        *self.state.write().unwrap() = State::Uninstalled;
        Err(InstallError::DesktopShortcutNotFound)
    }

    /// Uninstalls the game. WARNING: REMOVES ALL FILES FROM THE GAMES INSTALLATION DIRECTORY.
    pub async fn uninstall(&self) -> anyhow::Result<()> {
        let path = {
            let guard = self.state.read().unwrap();
            match *guard {
                State::Installed { ref path, .. } => Some(path.clone()),
                _ => None,
            }
        };

        if let Some(path) = path {
            // the game will still uninstall even if the files fail to be removed
            _ = tokio::fs::remove_dir_all(path).await;
        }

        // remove leftover desktop shortcuts (ignoring the error if they dont exist)
        _ = tokio::fs::remove_file(applications().join(format!("{}.desktop", self.slug))).await;
        _ = tokio::fs::remove_file(desktop().join(format!("{}.desktop", self.slug))).await;

        *self.state.write().unwrap() = State::Uninstalled;

        Ok(())
    }

    /// Is the game installed.
    pub fn installed(&self) -> bool {
        matches!(*self.state.read().unwrap(), State::Installed { .. })
    }

    /// Launch the game, if its installed.
    pub async fn play(&self) -> anyhow::Result<()> {
        let exe = {
            let mut state = self.state.write().unwrap();
            if let State::Installed {
                ref exe,
                ref mut is_open,
                ..
            } = *state
            {
                *is_open = true;
                Some(exe.clone())
            } else {
                None
            }
        };

        if let Some(exe_path) = exe {
            let start = Instant::now();

            umu(&[&exe_path.display().to_string()]).await?;

            let elapsed = start.elapsed();

            let mut state = self.state.write().unwrap();
            if let State::Installed {
                ref mut time_played,
                ref mut is_open,
                ..
            } = *state
            {
                *time_played += elapsed;
                *is_open = false;
            }
        }

        Ok(())
    }

    /// Extract the icon from a games executable, using `wine` if the game has a .lnk executable
    /// and `ico` utils if the executable is a regular `.exe` file.
    /// Errors if the game is not installed.
    pub async fn generate_icon(&self) -> anyhow::Result<PathBuf> {
        let exe = {
            let guard = self.state.read().unwrap();
            if let State::Installed { ref exe, .. } = *guard {
                Some(exe.clone())
            } else {
                None
            }
        };

        if let Some(exe) = exe {
            let path = icons().join(format!("{}.ico", self.slug));
            if exe
                .extension()
                .is_some_and(|ext| ext.eq_ignore_ascii_case("lnk"))
            {
                umu(&[
                    "winemenubuilder",
                    "-t",
                    &exe.display().to_string(),
                    &path.display().to_string(),
                ])
                .await?;
            } else {
                run(
                    "wrestool",
                    &[
                        "-x",
                        "-t14",
                        "--output",
                        &path.display().to_string(),
                        &exe.display().to_string(),
                    ],
                )
                .await?;
            }
            Ok(path)
        } else {
            Err(anyhow::anyhow!("Game is not installed!"))
        }
    }

    /// Makes a desktop shortcut in `~/.local/share/applications` and the users `Desktop` directory.
    /// The one in the `Desktop` directory will be a symlink to the one in the `applications` directory.
    ///
    /// Fails if the game doesn't have metadata.
    pub async fn make_shortcut(&self) -> anyhow::Result<()> {
        if let Some(meta) = &self.metadata {
            let exe = std::env::current_exe()?.display().to_string(); // Current exe path of freebie
            let icon = self.generate_icon().await?.display().to_string();
            let name = &meta.name;
            let slug = &self.slug;

            let shortcut = format!(
                "[Desktop Entry]
Type=Application
Name={name}
Comment=
Icon={icon}
Exec={exe} --launch={slug}
Categories=Game;
StartupNotify=true
Terminal=false",
            );

            let desktop_path = desktop().join(format!("{slug}.desktop"));
            let apps_path = applications().join(format!("{slug}.desktop"));

            tokio::fs::write(&apps_path, &shortcut).await?;
            tokio::fs::symlink(&apps_path, &desktop_path).await?;

            Ok(())
        } else {
            Err(anyhow::anyhow!("Game doesn't have metadata!"))
        }
    }
}

/// Search for video games on `fitgirl-repacks.site` and collect the results.
pub async fn search(query: &str) -> anyhow::Result<HashMap<String, Game>> {
    let mut games = HashMap::new();

    let url = format!("https://fitgirl-repacks.site/?s={query}");

    let html = reqwest::get(url).await?.text().await?;
    let document = Html::parse_document(&html);

    for article in document.select(&Selector::parse(".post").unwrap()) {
        let parse = || -> Option<Game> {
            let link = article
                .select(&Selector::parse(".entry-title > a").unwrap())
                .next()?;
            let summary = article
                .select(&Selector::parse(".entry-summary > p").unwrap())
                .next()?;

            let summary = summary.inner_html();

            if !summary.contains("Original Size: ") {
                return None;
            }

            let size: String = summary
                .split("Original Size: ")
                .nth(1)
                .unwrap()
                .split(' ')
                .take(2)
                .collect();

            let link = link.attr("href")?.to_string();
            let slug = link.split('/').nth(3).unwrap().to_string();

            Some(Game {
                metadata: None,
                link,
                size,
                slug,
                state: Arc::new(RwLock::new(State::default())),
            })
        };

        if let Some(game) = parse() {
            games.insert(game.slug.clone(), game);
        }
    }

    Ok(games)
}

pub fn popular() -> anyhow::Result<Vec<Arc<Game>>> {
    Ok(serde_json::from_str(include_str!("../popular.json"))?)
}
