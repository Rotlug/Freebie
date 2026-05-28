use clap::Parser;

/// Command line arguments
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Args {
    /// Slug of the game to launch
    #[arg(short, long)]
    pub launch: Option<String>,

    /// Slug of the game to download & install
    #[arg(short, long)]
    pub obtain: Option<String>,

    /// Client id and client secret for igdb, seperated by a comma.
    #[arg(short, long)]
    pub credentials: Option<String>,
}
