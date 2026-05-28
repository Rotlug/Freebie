use adw::prelude::*;
use relm4::{
    binding::{BoolBinding, ConnectBindingExt},
    prelude::*,
};
use std::sync::Arc;

use crate::{
    ActiveGames, TextureCache,
    game::{self, Game},
    igdb::MetadataManager,
    ui::game_button::{self, GameButton},
};

pub struct BrowseView {
    /// The `GameButton`s currently visible on the page
    game_buttons: AsyncFactoryVecDeque<GameButton>,
    /// Is the searching spinner currently revealed
    search_ongoing: BoolBinding,
    /// Mutable `HashMap` of all active games
    active_games: ActiveGames,
    /// Used for fetching the metadata of games
    metadata: Arc<MetadataManager>,
    /// The `JoinHandle` for the ongoing search, if there is one.
    current_search: Option<tokio::task::JoinHandle<()>>,
    /// Forwarded to the game buttons to not donwload covers twice
    texture_cache: TextureCache,
}

#[derive(Debug)]
pub enum Inbox {
    NewQuery(String),
    ShowPopular,
}

#[derive(Debug)]
pub enum Outbox {
    GameSelected(Arc<Game>, gtk::gdk::Texture),
}

#[derive(Debug)]
pub enum Command {
    SearchFinished(Vec<Arc<Game>>),
}

#[relm4::component(pub async)]
impl AsyncComponent for BrowseView {
    type Input = Inbox;
    type Output = Outbox;
    type Init = (ActiveGames, Arc<MetadataManager>, TextureCache);
    type CommandOutput = Command;

    view! {
        #[name = "browse_view"]
        gtk::Box {
            set_orientation: gtk::Orientation::Vertical,

            gtk::Revealer::with_binding(&model.search_ongoing) {
                #[name = "spinner_box"]
                gtk::Box {
                    set_orientation: gtk::Orientation::Horizontal,
                    set_halign: gtk::Align::Center,
                    set_valign: gtk::Align::Center,
                    set_margin_bottom: 7,

                    adw::Spinner {
                        set_width_request: 15,
                        set_margin_end: 5
                    },

                    gtk::Label {
                        set_label: "Searching...",
                        set_css_classes: &["dimmed"]
                    }
                },
            },

            gtk::ScrolledWindow {
                set_hexpand: true,
                set_vexpand: true,
                set_child = Some(model.game_buttons.widget()),
            }
        }
    }

    async fn init(
        init: Self::Init,
        root: Self::Root,
        sender: AsyncComponentSender<Self>,
    ) -> AsyncComponentParts<Self> {
        let flow_box = gtk::FlowBox::builder()
            .selection_mode(gtk::SelectionMode::None)
            .homogeneous(true)
            .margin_start(10)
            .margin_bottom(10)
            .margin_top(10)
            .margin_end(10)
            .halign(gtk::Align::Center)
            .valign(gtk::Align::Start)
            .row_spacing(12)
            .column_spacing(12)
            .build();

        let game_buttons = AsyncFactoryVecDeque::builder().launch(flow_box).forward(
            sender.output_sender(),
            |msg| match msg {
                game_button::Outbox::Clicked(game, texture) => Outbox::GameSelected(game, texture),
            },
        );

        let (active_games, metadata, texture_cache) = init;
        let model = Self {
            active_games,
            metadata,
            game_buttons,
            search_ongoing: BoolBinding::default(),
            current_search: None,
            texture_cache,
        };

        let widgets = view_output!();

        AsyncComponentParts { model, widgets }
    }

    async fn update(
        &mut self,
        message: Self::Input,
        sender: AsyncComponentSender<Self>,
        _root: &Self::Root,
    ) {
        match message {
            Inbox::NewQuery(query) => {
                self.search_ongoing.set_value(true);

                let command = sender.command_sender().clone();
                let metadata = self.metadata.clone();
                let handle = tokio::spawn(async move {
                    if let Ok(games) = search(&metadata, &query).await {
                        _ = command.send(Command::SearchFinished(games));
                    }
                });

                self.current_search = Some(handle);
            }
            Inbox::ShowPopular => {
                if let Ok(popular) = game::popular() {
                    sender
                        .command_sender()
                        .send(Command::SearchFinished(popular))
                        .unwrap();
                }
            }
        }
    }

    async fn update_cmd(
        &mut self,
        message: Self::CommandOutput,
        _sender: AsyncComponentSender<Self>,
        _root: &Self::Root,
    ) {
        match message {
            Command::SearchFinished(games) => {
                self.search_ongoing.set_value(false);
                let mut guard = self.game_buttons.guard();
                let active_games = self.active_games.lock().unwrap();
                guard.clear();
                for game in games {
                    guard.push_back(if let Some(active) = active_games.get(&game.slug) {
                        (active.clone(), self.texture_cache.clone())
                    } else {
                        (game, self.texture_cache.clone())
                    });
                }
            }
        }
    }
}

/// Fetch games with their metadata
async fn search(metadata: &MetadataManager, query: &str) -> anyhow::Result<Vec<Arc<Game>>> {
    let mut games = game::search(query).await?;
    let slugs: Vec<&String> = games.keys().collect();
    let mut metas = metadata.get_games(&slugs).await?;

    // Map metadata to game
    for (game_slug, game) in &mut games {
        if let Some(matching) = metas.remove(game_slug) {
            game.metadata = Some(matching);
        }
    }

    // Filter out games which don't have metadata
    Ok(games
        .into_values()
        .map(Arc::new)
        .filter(|game| game.metadata.is_some())
        .collect())
}
