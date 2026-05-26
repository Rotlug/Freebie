use std::sync::Arc;

use adw::prelude::*;
use relm4::prelude::*;

use crate::{
    ActiveGames, TextureCache,
    game::Game,
    ui::game_button::{self, GameButton},
};

#[derive(Debug)]
pub enum Inbox {
    Update,
    SearchStarted(String),
}

#[derive(Debug)]
pub enum Outbox {
    GameSelected(Arc<Game>, gtk::gdk::Texture),
}

pub struct PlayView {
    active_games: ActiveGames,
    texture_cache: TextureCache,
    game_buttons: AsyncFactoryVecDeque<GameButton>,
    filter: String,
}

#[relm4::component(pub async)]
impl AsyncComponent for PlayView {
    type Init = (ActiveGames, TextureCache);
    type Input = Inbox;
    type Output = Outbox;
    type CommandOutput = ();

    view! {
        #[name = "play_view"]
        gtk::Box {
            set_orientation: gtk::Orientation::Vertical,

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
        sender: relm4::AsyncComponentSender<Self>,
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

        let (active_games, texture_cache) = init;
        let model = Self {
            active_games,
            texture_cache,
            game_buttons,
            filter: String::new(),
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
            Inbox::Update => {
                let mut guard = self.game_buttons.guard();
                let installed_games: Vec<Arc<Game>> = self
                    .active_games
                    .lock()
                    .unwrap()
                    .values()
                    .filter(|g| g.slug.contains(&self.filter) && g.installed())
                    .map(Arc::clone)
                    .collect();

                guard.clear();
                for game in installed_games {
                    guard.push_back((game, self.texture_cache.clone()));
                }
            }
            Inbox::SearchStarted(query) => {
                self.filter = query;
                sender.input(Inbox::Update);
            }
        }
    }
}
