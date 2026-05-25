use std::sync::Arc;

use crate::{
    ActiveGames,
    game::Game,
    igdb::{self, MetadataManager},
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
    browse_view: AsyncController<BrowseView>,
    search_visible: BoolBinding,
    active_view: View,
}

#[derive(Debug)]
pub enum View {
    Browse,
}

#[derive(Debug)]
pub enum Outbox {
    GameSelected(Arc<Game>, gtk::gdk::Texture),
}

#[derive(Debug)]
pub enum Inbox {
    SearchStarted(String),
    SearchBarEmpty,
    ViewChanged(View),
}

#[relm4::component(pub async)]
impl AsyncComponent for MainPage {
    type Input = Inbox;
    type Output = Outbox;
    type Init = (adw::Window, ActiveGames);
    type CommandOutput = ();

    view! {
        adw::ToolbarView {
            add_top_bar = &adw::HeaderBar {
                pack_start = &gtk::ToggleButton::with_binding(&model.search_visible) {
                    set_icon_name: "system-search-symbolic",
                },

                #[wrap(Some)]
                #[name="view_switcher"]
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
                add_binding: (&model.search_visible, "search-mode-enabled"),

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

    async fn init(
        init: Self::Init,
        root: Self::Root,
        sender: AsyncComponentSender<Self>,
    ) -> AsyncComponentParts<Self> {
        let active_games = init.1;
        let metadata = Arc::new(MetadataManager::new(igdb::Credentials {
            client_id: "7c9e7z9nn822m4y00n0xkmwch6y2mu".into(),
            client_secret: "fydcw9o03z77uvckldtlzdz0qyxetf".into(),
        }));

        let browse_view = BrowseView::builder()
            .launch((active_games.clone(), metadata.clone()))
            .forward(sender.output_sender(), |msg| match msg {
                browse_view::Outbox::GameSelected(game, texture) => {
                    Outbox::GameSelected(game, texture)
                }
            });

        let search_visible = BoolBinding::default();
        let model = Self {
            root_window: init.0,
            browse_view,
            search_visible,
            active_view: View::Browse,
        };

        let widgets = view_output!();
        widgets.search_bar.connect_entry(&widgets.search_entry);

        // Search signals
        let inbox = sender.input_sender().clone();
        widgets.search_entry.connect_search_changed(move |entry| {
            let query = entry.text().to_string();
            inbox
                .send(match query.len() {
                    0 => Inbox::SearchBarEmpty,
                    3.. => Inbox::SearchStarted(query),
                    _ => return,
                })
                .unwrap();
        });

        // View switcher signals
        let inbox = sender.input_sender().clone();
        widgets
            .stack
            .connect_visible_child_name_notify(move |stack| {
                if let Some(visible_child) = stack.visible_child_name() {
                    inbox
                        .send(match visible_child.as_str() {
                            "browse" => Inbox::ViewChanged(View::Browse),
                            _ => return,
                        })
                        .unwrap();
                }
            });

        AsyncComponentParts { model, widgets }
    }

    async fn update(
        &mut self,
        message: Self::Input,
        _sender: AsyncComponentSender<Self>,
        _root: &Self::Root,
    ) {
        match message {
            Inbox::ViewChanged(view) => self.active_view = view,
            Inbox::SearchStarted(query) => match self.active_view {
                View::Browse => {
                    self.browse_view
                        .emit(browse_view::Inbox::SearchStarted(query));
                }
            },
            Inbox::SearchBarEmpty => match self.active_view {
                View::Browse => {
                    self.browse_view.emit(browse_view::Inbox::SearchBarEmpty);
                }
            },
        }
    }
}
