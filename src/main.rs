use clap::Parser;
use relm4::prelude::*;

use crate::{
    app::App,
    args::Args,
    util::{ensure_directories_exist, installed_games, installed_games_file},
};

mod app;
mod args;
mod error;
mod game;
mod igdb;
mod preferences;
mod ui;
mod util;

fn main() {
    ensure_directories_exist();
    let args = Args::parse();
    if let Some(slug) = args.game {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();

        rt.block_on(async move {
            let installed = installed_games().await?;
            if let Some(game) = installed.get(&slug) {
                if let Err(e) = game.play().await {
                    eprintln!("Error launching game: {e}");
                }

                let string = serde_json::to_string(&installed)?;
                tokio::fs::write(installed_games_file(), &string).await?;
            } else {
                eprintln!("Game not found: {slug}");
                eprintln!("Installed games:");
                for game in installed.values() {
                    eprintln!("{}", game.slug);
                }
            }

            Ok::<(), anyhow::Error>(())
        })
        .unwrap();
    } else {
        let app = RelmApp::new("land.lugasi.freebie");
        app.run_async::<App>(());
    }
}
