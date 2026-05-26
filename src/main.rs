use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use adw::prelude::*;
use relm4::prelude::*;

use crate::{
    game::Game,
    ui::{
        game_page::{self, GamePage},
        main_page::{self, MainPage},
    },
    util::{ensure_directories_exist, installed_games, installed_games_file},
};

mod error;
mod util;

mod game;
mod igdb;
mod ui;

/// "Active" games are games that the user has interacted with during the runtime of the program.
/// Therefore, we need to keep track of them so their state doesn't get lost when
/// reentering the game page, canceling a search, etc..
pub type ActiveGames = Arc<Mutex<HashMap<String, Arc<Game>>>>;

/// Keep downloaded textures in this structure to prevent downloading textures twice
pub type TextureCache = Arc<Mutex<HashMap<String, gtk::gdk::Texture>>>;

struct App {
    main_page: AsyncController<MainPage>,
    game_page: AsyncController<GamePage>,
    active_games: ActiveGames,
}

#[derive(Debug)]
pub enum Inbox {
    GameSelected(Arc<Game>, gtk::gdk::Texture),
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
            set_size_request: (900, 500),

            #[name = "nav_view"]
            adw::NavigationView {
                #[name = "nav_main_page"]
                add = &adw::NavigationPage {
                    set_child = Some(model.main_page.widget()),
                },

                #[name = "nav_game_page"]
                add = &adw::NavigationPage {
                  set_child = Some(model.game_page.widget()),
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

        let active_games = Arc::new(Mutex::new(
            installed_games()
                .await
                .inspect_err(|g| {
                    _ = dbg!(g);
                })
                .unwrap(),
        ));

        let texture_cache = Arc::new(Mutex::new(HashMap::new()));

        let main_page = MainPage::builder()
            .launch((root.clone(), active_games.clone(), texture_cache.clone()))
            .forward(sender.input_sender(), |msg| match msg {
                main_page::Outbox::GameSelected(game, texture) => {
                    Inbox::GameSelected(game, texture)
                }
            });
        let game_page = GamePage::builder().launch(active_games.clone()).detach();

        let model = Self {
            main_page,
            game_page,
            active_games,
        };
        let widgets = view_output!();

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
        _sender: AsyncComponentSender<Self>,
        _root: &Self::Root,
    ) {
        match message {
            Inbox::GameSelected(game, texture) => {
                self.game_page
                    .emit(game_page::Inbox::ChangeGame(game, texture));

                widgets.nav_view.push(&widgets.nav_game_page);
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
    let app = RelmApp::new("land.lugasi.freebie");
    app.run_async::<App>(());
}
