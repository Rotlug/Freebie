use std::sync::{Arc, RwLock};

use clap::Parser;
use relm4::prelude::*;

use crate::{
    app::App,
    args::Args,
    igdb::MetadataManager,
    preferences::PreferencesInner,
    util::{
        downloads, ensure_directories_exist, installed_games, installed_games_file, preferences,
        slug::SlugExt,
    },
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

    // no args
    if std::env::args().len() == 1 {
        let app = RelmApp::new("land.lugasi.freebie");
        app.run_async::<App>(());
        return;
    }

    spawn(async move {
        let args = Args::parse();
        if let Some(game) = args.obtain {
            // Try to make metadata manager from command line args OR preferences file
            let metadata = if let Some(creds) = args.credentials {
                let mut split = creds.split(',');
                let credentials = igdb::Credentials {
                    client_id: split.next().expect("Not enough credentials").to_string(),
                    client_secret: split.next().expect("Not enough credentials").to_string(),
                };
                let preferences = Arc::new(RwLock::new(PreferencesInner { credentials }));
                MetadataManager::new(preferences)
            } else {
                let preferences = preferences().await.expect("Preferences file doesn't exist or is malformed, pass --credentials or launch freebie in interactive mode at least once.");
                MetadataManager::new(Arc::new(RwLock::new(preferences)))
            };

            obtain(&game.slug(), &metadata).await;
        }

        if let Some(game) = args.launch {
            launch(&game.slug()).await;
        }
    });
}

fn spawn<F: Future>(block: F) -> F::Output {
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();

    rt.block_on(block)
}

/// Launch game, if its in the `installed_games` file.
async fn launch(slug: &str) {
    let installed_games = installed_games().await.unwrap_or_else(|_| {
        panic!(
            "Installed games file at {} is malformed",
            installed_games_file().display()
        )
    });

    let Some(game) = installed_games.get(slug) else {
        eprintln!("Game not found: {slug}. Installed games are:");
        for (slug, _) in installed_games {
            eprintln!("{slug}");
        }
        return;
    };

    _ = game.play().await;
}

/// Download and install a game
async fn obtain(slug: &str, metadata: &MetadataManager) {
    println!("Searching for {slug}");
    let mut games = game::search(slug)
        .await
        .unwrap_or_else(|err| panic!("Failed to perform online search: {err:?}"));

    let Some(mut game) = games.remove(slug) else {
        eprintln!("Couldn't find game {slug} in search. Found games:");
        for (slug, _) in games {
            eprintln!("{slug}");
        }
        return;
    };

    println!("Getting metadata");
    let mut metas = metadata
        .get_games(&[slug])
        .await
        .expect("Failed to fetch metadatas for game");
    let Some(meta) = metas.remove(slug) else {
        panic!("Couldn't find metadata for game {slug}");
    };
    println!("{meta}");
    game.metadata = Some(meta);

    let session = librqbit::Session::new(downloads())
        .await
        .expect("Failed to create torrent session!");

    println!("File size: {}", game.size);

    let download = game
        .download(&session, true)
        .await
        .unwrap_or_else(|err| panic!("Failed to download game: {err:?}"));
    println!();

    println!("Installing");
    _ = game
        .install(&download)
        .await
        .unwrap_or_else(|err| panic!("Failed to install game: {err:?}"));

    let mut installed_games = installed_games().await.unwrap_or_else(|_| {
        panic!(
            "Installed games file at {} is malformed",
            installed_games_file().display()
        )
    });
    installed_games.insert(slug.to_string(), Arc::new(game));

    tokio::fs::write(
        installed_games_file(),
        serde_json::to_string(&installed_games).unwrap(),
    )
    .await
    .unwrap();
}
