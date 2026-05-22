//! Errors thrown by various actions in `freebie`

use thiserror::Error;

/// An error that happens while downloading a game
#[derive(Error, Debug)]
pub enum DownloadError {
    #[error("Network or HTTP Error: {0}")]
    NetworkError(#[from] reqwest::Error),

    #[error("Magnet link not found on game's web page")]
    MagnetLinkNotFound,

    #[error("Torrent management failed: {0}")]
    TorrentError(&'static str),
}

/// An error that happens while installing a game
#[derive(Error, Debug)]
pub enum InstallError {
    #[error("IO Error: {0}")]
    IOError(#[from] tokio::io::Error),

    #[error("Setup.exe was not found")]
    SetupExeNotFound,

    #[error("Desktop shortcut not found")]
    DesktopShortcutNotFound,
}
