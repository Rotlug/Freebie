use std::sync::Arc;

use crate::{
    game::Game,
    ui::browse_view::{self, BrowseView},
};
use adw::prelude::*;
use relm4::{
    RelmObjectExt,
    binding::{BoolBinding, ConnectBindingExt},
    prelude::*,
};

pub struct MainPage {
    root_window: adw::Window,
    browse_view: Controller<BrowseView>,
    search_enabled: BoolBinding,
}

#[derive(Debug)]
pub enum View {
    Browse,
    Play,
}

#[derive(Debug)]
pub enum Outbox {
    NewSearch(String),
    ChangeView(View),
}

#[derive(Debug)]
pub enum Inbox {
    ReceivedGames(Vec<Arc<Game>>),
}

#[relm4::component(pub)]
impl SimpleComponent for MainPage {
    type Input = Inbox;
    type Output = Outbox;
    type Init = adw::Window;

    view! {
        adw::ToolbarView {
            add_top_bar = &adw::HeaderBar {
                pack_start = &gtk::ToggleButton::with_binding(&model.search_enabled) {
                    set_icon_name: "system-search-symbolic",
                },

                #[wrap(Some)]
                set_title_widget = &adw::ViewSwitcher {
                    set_policy: adw::ViewSwitcherPolicy::Wide,
                    set_stack: Some(&stack),
                },
            },

            #[wrap(Some)]
            #[name="stack"]
            set_content = &adw::ViewStack {
                add = &gtk::Box {
                    append = model.browse_view.widget(),
                } -> {
                    set_name: Some("browse"),
                    set_title: Some("Browse"),
                    set_icon_name: Some("view-grid-symbolic"),
                },
            },

            #[name="search_bar"]
            add_top_bar = &gtk::SearchBar {
                set_key_capture_widget: Some(&model.root_window),
                add_binding: (&model.search_enabled, "search-mode-enabled"),

                #[wrap(Some)]
                set_child = &adw::Clamp {
                    set_maximum_size: 300,

                    #[name="search_entry"]
                    gtk::SearchEntry {
                        set_hexpand: true,
                        set_placeholder_text: Some("Search games..."),
                        set_search_delay: 300,
                    }
                }
            }
        }
    }

    fn init(
        init: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let browse_view = BrowseView::builder()
            .launch(())
            .forward(sender.input_sender(), |msg| match msg {
                browse_view::Outbox::GameSelected(game) => todo!(),
            });
        let search_enabled = BoolBinding::default();
        let model = Self {
            root_window: init,
            browse_view,
            search_enabled,
        };

        let widgets = view_output!();
        widgets.search_bar.connect_entry(&widgets.search_entry);

        let sender = sender.output_sender().clone();
        widgets.search_entry.connect_search_changed(move |entry| {
            let query = entry.text().to_string();
            if query.len() >= 3 {
                sender.send(Outbox::NewSearch(query)).unwrap();
            }
        });

        ComponentParts { model, widgets }
    }

    fn update(&mut self, message: Self::Input, sender: ComponentSender<Self>) {
        match message {
            Inbox::ReceivedGames(games) => {
                self.browse_view
                    .emit(browse_view::Inbox::ReceivedGames(games));
            }
        }
    }
}
