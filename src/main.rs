use std::{collections::HashMap, sync::Arc};

use adw::prelude::*;
use relm4::prelude::*;

use crate::{
    game::Game,
    igdb::MetadataManager,
    ui::main_page::{self, MainPage},
};

mod error;
mod util;

mod game;
mod igdb;
mod ui;

struct App {
    main_page: Controller<MainPage>,
    metadata_manager: Arc<MetadataManager>,
    current_search: Option<tokio::task::JoinHandle<anyhow::Result<()>>>,
}

#[derive(Debug)]
enum Outbox {
    SendMetadata(HashMap<String, igdb::Metadata>),
}

#[derive(Debug)]
enum Inbox {
    SearchTriggered(String),
    MetadataRequest(Vec<String>),
}

#[derive(Debug)]
enum Command {
    SearchFinished(Vec<Arc<Game>>),
}

#[relm4::component(async)]
impl AsyncComponent for App {
    type Output = Outbox;
    type Input = Inbox;
    type Init = ();
    type CommandOutput = Command;

    view! {
        adw::Window {
            set_size_request: (900, 500),

            adw::NavigationView {
                #[name = "nav_main_page"]
                add = &adw::NavigationPage {
                    set_child = Some(model.main_page.widget()),
                }
            }
        }
    }

    async fn init(
        init: Self::Init,
        root: Self::Root,
        sender: AsyncComponentSender<Self>,
    ) -> AsyncComponentParts<Self> {
        let metadata_manager = Arc::new(MetadataManager::new(igdb::Credentials {
            client_id: "7c9e7z9nn822m4y00n0xkmwch6y2mu".into(),
            client_secret: "fydcw9o03z77uvckldtlzdz0qyxetf".into(),
        }));

        let main_page = MainPage::builder().launch(root.clone()).forward(
            sender.input_sender(),
            |msg| match msg {
                ui::main_page::Outbox::NewSearch(query) => Inbox::SearchTriggered(query),
                ui::main_page::Outbox::ChangeView(view) => todo!(),
            },
        );

        let model = Self {
            main_page,
            metadata_manager,
            current_search: None,
        };

        let widgets = view_output!();

        AsyncComponentParts { model, widgets }
    }

    async fn update(
        &mut self,
        message: Self::Input,
        sender: AsyncComponentSender<Self>,
        root: &Self::Root,
    ) {
        match message {
            Inbox::SearchTriggered(query) => {
                // cancel previous search
                if let Some(handle) = self.current_search.take() {
                    handle.abort();
                }

                let sender = sender.command_sender().clone();
                let metadata_manager = self.metadata_manager.clone();

                let new_handle = tokio::spawn(async move {
                    let mut games = game::search(&query).await?;

                    let slugs: Vec<String> = games.keys().cloned().collect();
                    let metas = metadata_manager.get_games(&slugs).await?;

                    // Map metadata to games
                    for meta in metas.into_values() {
                        if let Some(game) = games.get_mut(&meta.slug) {
                            game.metadata = Some(meta);
                        }
                    }

                    sender
                        .send(Command::SearchFinished(
                            games
                                .into_values()
                                .filter(|g| g.metadata.is_some())
                                .map(Arc::new)
                                .collect(),
                        ))
                        .unwrap();
                    Ok::<(), anyhow::Error>(())
                });

                self.current_search = Some(new_handle);
            }

            Inbox::MetadataRequest(slugs) => {}
        }
    }

    async fn update_cmd(
        &mut self,
        message: Self::CommandOutput,
        sender: AsyncComponentSender<Self>,
        root: &Self::Root,
    ) {
        match message {
            Command::SearchFinished(games) => {
                self.current_search = None;

                self.main_page.emit(main_page::Inbox::ReceivedGames(games));
            }
        }
    }
}

fn main() {
    let app = RelmApp::new("land.lugasi.freebie");
    app.run_async::<App>(());
}
