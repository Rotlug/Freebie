//! Search, download and install games from `fitgirl-repacks.site`

use librqbit::{AddTorrent, AddTorrentOptions};
use scraper::{Html, Selector};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    fs,
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
    time::Duration,
};
use tokio::{self};

use crate::{
    error::{DownloadError, InstallError},
    igdb,
    util::{downloads, set_prefix_mute, slug::SlugExt, umu, wine_desktop, wine_games},
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

    /// The games current installation state
    #[serde(default)]
    pub state: Arc<Mutex<State>>,
}

/// A Games current installation state
#[derive(Default, Debug, Serialize, Deserialize)]
pub enum State {
    /// The game is uninstalled
    #[default]
    Uninstalled,
    /// The game is currently in the middle of being downloaded. the
    /// `String` contains a human-readable version of the current percentage.
    /// (for example: `"25.46%"`)
    Downloading(String),
    /// The game's installer is currently running.
    Installing,
    /// The game is already installed
    Installed {
        /// The game's installation directory. usually this is the directory above
        /// The game's `exe` location.
        path: PathBuf,
        /// The location for the games `exe` or `lnk` file.
        exe: PathBuf,
    },
}

impl Game {
    /// Download a game using `link`. Returns the output directory of the download.
    pub async fn download(
        &self,
        session: &Arc<librqbit::Session>,
    ) -> Result<PathBuf, DownloadError> {
        let html = reqwest::get(&self.link).await?.text().await?;
        let document = Html::parse_document(&html);

        // Find first magnet link in page
        let magnet = document
            .select(&Selector::parse("a").unwrap())
            .find_map(|link| {
                let href = link.attr("href")?;

                if href.starts_with("magnet") {
                    return Some(href);
                }

                None
            })
            .ok_or(DownloadError::MagnetLinkNotFound)?;

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

                dbg!(&progress);
                *self.state.lock().unwrap() = State::Downloading(progress);
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
        *self.state.lock().unwrap() = State::Installing;
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

        // Look for a matching .desktop shortcut
        for entry in fs::read_dir(wine_desktop())?.flatten() {
            let Ok(file_name) = entry.file_name().into_string() else {
                continue;
            };
            let Some(file_name) = file_name.strip_suffix(".lnk") else {
                continue;
            };

            if self.slug.contains(&file_name.ultra_slug()) {
                *self.state.lock().unwrap() = State::Installed {
                    path: wine_games().join(&self.slug),
                    exe: entry.path(),
                };
                return Ok(entry.path());
            }
        }

        // No Desktop shortcut has been found, installation failed.
        *self.state.lock().unwrap() = State::Uninstalled;
        Err(InstallError::DesktopShortcutNotFound)
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
                state: Arc::new(Mutex::new(State::default())),
            })
        };

        if let Some(game) = parse() {
            games.insert(game.slug.clone(), game);
        }
    }

    Ok(games)
}
