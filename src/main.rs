use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use adw::prelude::*;
use clap::Parser;
use relm4::prelude::*;

use crate::{
    args::Args,
    game::Game,
    settings::Settings,
    ui::{
        game_page::{self, GamePage},
        main_page::{self, MainPage},
        welcome_page::{self, WelcomePage},
    },
    util::{
        ensure_directories_exist, installed_games, installed_games_file, settings, settings_file,
    },
};

mod args;
mod error;
mod game;
mod igdb;
mod settings;
mod ui;
mod util;

/// "Active" games are games that the user has interacted with during the runtime of the program.
/// Therefore, we need to keep track of them so their state doesn't get lost when
/// reentering the game page, canceling a search, etc..
pub type ActiveGames = Arc<Mutex<HashMap<String, Arc<Game>>>>;

/// Keep downloaded textures in this structure to prevent downloading textures twice
pub type TextureCache = Arc<Mutex<HashMap<String, gtk::gdk::Texture>>>;

struct App {
    main_page: Option<AsyncController<MainPage>>,
    game_page: AsyncController<GamePage>,
    welcome_page: Option<AsyncController<WelcomePage>>,
    active_games: ActiveGames,
    settings: Option<Settings>,
}

#[derive(Debug)]
pub enum Inbox {
    GameSelected(Arc<Game>, gtk::gdk::Texture),
    WelcomeDone(Settings),
    GameUninstalled(Arc<Game>),
    Exit,
}

#[relm4::component(async)]
impl AsyncComponent for App {
    type Output = ();
    type Input = Inbox;
    type Init = ();
    type CommandOutput = ();

    view! {
      adw::Window {
          #[name = "stack"]
          adw::ViewStack {
              set_enable_transitions: true,

              #[name = "welcome"]
              add_titled[Some("welcome"), "Welcome"] = &adw::Bin {
              },

              #[name = "nav_view"]
              add_titled[Some("main"), "Main"] = &adw::NavigationView {
                  #[name = "nav_main_page"]
                  add = &adw::NavigationPage {
                  },

                  #[name = "nav_game_page"]
                  add = &adw::NavigationPage {
                      set_child: Some(model.game_page.widget()),
                  }
              }
            }
        }
    }

    async fn init(
        _init: Self::Init,
        root: Self::Root,
        sender: AsyncComponentSender<Self>,
    ) -> AsyncComponentParts<Self> {
        ensure_directories_exist().await;
        let active_games = Arc::new(Mutex::new(installed_games().await.unwrap()));
        let texture_cache = Arc::new(Mutex::new(HashMap::new()));

        let settings = settings().await.ok();
        let has_settings = settings.is_some();

        // Because the main page needs the settings we need to add it only if the settings exist.
        // If not then we show the welcome page and only then we add the main page to the nav view.
        let main_page = if let Some(ref current_settings) = settings {
            let controller = MainPage::builder()
                .launch((
                    root.clone(),
                    active_games.clone(),
                    texture_cache.clone(),
                    current_settings.clone(),
                ))
                .forward(sender.input_sender(), |msg| match msg {
                    main_page::Outbox::GameSelected(game, texture) => {
                        Inbox::GameSelected(game, texture)
                    }
                });
            Some(controller)
        } else {
            None
        };

        let game_page = GamePage::builder()
            .launch((active_games.clone(), root.clone()))
            .forward(sender.input_sender(), |msg| match msg {
                game_page::Outbox::GameUninstalled(game) => Inbox::GameUninstalled(game),
            });

        // If the settings dont exist or fail to load, show the welcome page
        // the welcome page is where the user inserts their credentials
        let welcome_page = if has_settings {
            None
        } else {
            Some({
                WelcomePage::builder()
                    .launch(())
                    .forward(sender.input_sender(), |msg| match msg {
                        welcome_page::Outbox::Done(settings) => Inbox::WelcomeDone(settings),
                    })
            })
        };

        let model = Self {
            main_page,
            game_page,
            welcome_page,
            active_games,
            settings,
        };

        let widgets = view_output!();

        if has_settings {
            if let Some(ref mp) = model.main_page {
                widgets.nav_main_page.set_child(Some(mp.widget()));
            }
            root.set_size_request(900, 500);
            root.set_resizable(true);
            widgets.stack.set_visible_child(&widgets.nav_view);
        } else {
            if let Some(ref welcome) = model.welcome_page {
                widgets.welcome.set_child(Some(welcome.widget()));
            }
            root.set_size_request(450, 300);
            root.set_resizable(false);
            widgets.stack.set_visible_child(&widgets.welcome);
        }

        if let Some(app) = root.application() {
            let inbox = sender.input_sender().clone();
            app.connect_shutdown(move |_| inbox.clone().send(Inbox::Exit).unwrap());
        }
        AsyncComponentParts { model, widgets }
    }

    async fn update_with_view(
        &mut self,
        widgets: &mut Self::Widgets,
        message: Self::Input,
        sender: AsyncComponentSender<Self>,
        root: &Self::Root,
    ) {
        match message {
            Inbox::GameSelected(game, texture) => {
                self.game_page
                    .emit(game_page::Inbox::ChangeGame(game, texture));

                widgets.nav_view.push(&widgets.nav_game_page);
            }
            Inbox::WelcomeDone(settings) => {
                // Write settings to disk
                let string = serde_json::to_string(&settings).unwrap();
                tokio::fs::write(settings_file(), &string).await.unwrap();
                self.settings = Some(settings.clone());

                // Add main page to ui
                let main_page = MainPage::builder()
                    .launch((
                        root.clone(),
                        self.active_games.clone(),
                        Arc::new(Mutex::new(HashMap::new())),
                        settings,
                    ))
                    .forward(sender.input_sender(), |msg| match msg {
                        main_page::Outbox::GameSelected(game, texture) => {
                            Inbox::GameSelected(game, texture)
                        }
                    });
                widgets.nav_main_page.set_child(Some(main_page.widget()));
                self.main_page = Some(main_page);

                root.set_resizable(true);
                root.set_size_request(900, 500);
                widgets.stack.set_visible_child(&widgets.nav_view);
            }
            Inbox::GameUninstalled(game) => {
                self.active_games.lock().unwrap().remove(&game.slug);
                if let Some(ref main_page) = self.main_page {
                    main_page.emit(main_page::Inbox::GameUninstalled);
                }
            }
            Inbox::Exit => {
                // Save to disk only the state of games which finished installing
                let installed_games: HashMap<String, Arc<Game>> = {
                    let guard = self.active_games.lock().unwrap().clone();
                    guard
                        .into_iter()
                        .filter(|(_, game)| game.installed())
                        .collect()
                };

                if let Ok(string) = serde_json::to_string(&installed_games) {
                    tokio::fs::write(installed_games_file(), &string)
                        .await
                        .unwrap();
                }
            }
        }
    }
}

fn main() {
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
