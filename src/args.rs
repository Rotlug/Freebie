use clap::Parser;

/// Command line arguments
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Args {
    /// Id of the game to execute without launching the gui
    #[arg(short, long)]
    pub game: Option<String>,
}
