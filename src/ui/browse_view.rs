use adw::prelude::*;
use relm4::{binding::BoolBinding, prelude::*};
use std::sync::Arc;

use crate::{game::Game, ui::game_button::GameButton};

pub struct BrowseView {
    game_buttons: AsyncFactoryVecDeque<GameButton>,
    is_searching: BoolBinding,
}

#[derive(Debug)]
pub enum Inbox {
    ReceivedGames(Vec<Arc<Game>>),
}

#[derive(Debug)]
pub enum Outbox {
    GameSelected(Arc<Game>),
}

#[relm4::component(pub)]
impl SimpleComponent for BrowseView {
    type Input = Inbox;
    type Output = Outbox;
    type Init = ();

    view! {
        #[name = "browse_view"]
        gtk::Box {
            gtk::ScrolledWindow {
                set_hexpand: true,
                set_vexpand: true,
                set_child = Some(model.game_buttons.widget()),
            }
        }
    }

    fn init(
        init: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
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

        let game_buttons = AsyncFactoryVecDeque::builder().launch(flow_box).detach();
        let model = Self {
            game_buttons,
            is_searching: BoolBinding::default(),
        };
        let widgets = view_output!();

        ComponentParts { model, widgets }
    }

    fn update(&mut self, message: Self::Input, sender: ComponentSender<Self>) {
        match message {
            Inbox::ReceivedGames(games) => {
                let mut guard = self.game_buttons.guard();
                guard.clear();
                for game in games {
                    guard.push_back(game);
                }
            }
        }
    }
}
