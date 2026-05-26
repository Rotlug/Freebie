use std::sync::Arc;

use crate::{
    ActiveGames, TextureCache,
    game::Game,
    igdb::MetadataManager,
    settings::Settings,
    ui::{
        browse_view::{self, BrowseView},
        play_view::{self, PlayView},
    },
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
    play_view: AsyncController<PlayView>,
    search_visible: BoolBinding,
    active_view: View,
}

#[derive(Debug)]
pub enum View {
    Browse,
    Play,
}

#[derive(Debug)]
pub enum Outbox {
    GameSelected(Arc<Game>, gtk::gdk::Texture),
}

#[derive(Debug)]
pub enum Inbox {
    /// fires when the search is started properly, taking `search_delay` into account
    SearchStarted(String),
    /// fires every time the search entry text changes, disregarding the `search_delay`.
    SearchEntryUpdated(String),
    /// fires when the search bar is empty, taking `search_delay` into account.
    SearchBarEmpty,
    ViewChanged(View),
}

#[relm4::component(pub async)]
impl AsyncComponent for MainPage {
    type Input = Inbox;
    type Output = Outbox;
    type Init = (adw::Window, ActiveGames, TextureCache, Settings);
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

                add = &gtk::Box {
                    append = model.play_view.widget(),
                } -> {
                    set_name: Some("play"),
                    set_title: Some("Play"),
                    set_icon_name: Some("applications-games-symbolic")
                }
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
                        set_search_delay: 300
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
        let (root_window, active_games, texture_cache, settings) = init;
        let metadata = Arc::new(MetadataManager::new(settings.credentials));

        let browse_view = BrowseView::builder()
            .launch((
                active_games.clone(),
                metadata.clone(),
                texture_cache.clone(),
            ))
            .forward(sender.output_sender(), |msg| match msg {
                browse_view::Outbox::GameSelected(game, texture) => {
                    Outbox::GameSelected(game, texture)
                }
            });

        let play_view = PlayView::builder()
            .launch((active_games.clone(), texture_cache.clone()))
            .forward(sender.output_sender(), |msg| match msg {
                play_view::Outbox::GameSelected(game, texture) => {
                    Outbox::GameSelected(game, texture)
                }
            });

        let search_visible = BoolBinding::new(false);
        let model = Self {
            root_window,
            browse_view,
            play_view,
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

        let inbox = sender.input_sender().clone();
        widgets.search_entry.connect_changed(move |entry| {
            let query = entry.text().to_string();
            inbox.send(Inbox::SearchEntryUpdated(query)).unwrap();
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
                            "play" => Inbox::ViewChanged(View::Play),
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
        sender: AsyncComponentSender<Self>,
        _root: &Self::Root,
    ) {
        match message {
            Inbox::ViewChanged(view) => {
                self.search_visible.set_value(false);
                sender.input(Inbox::SearchEntryUpdated(String::new()));
                match view {
                    View::Play => self.play_view.emit(play_view::Inbox::Update),
                    View::Browse => self.browse_view.emit(browse_view::Inbox::SearchBarEmpty),
                }
                self.active_view = view;
            }
            Inbox::SearchStarted(query) => match self.active_view {
                View::Browse => self
                    .browse_view
                    .emit(browse_view::Inbox::SearchStarted(query)),
                View::Play => {}
            },
            Inbox::SearchBarEmpty => match self.active_view {
                View::Browse => self.browse_view.emit(browse_view::Inbox::SearchBarEmpty),
                View::Play => {}
            },
            Inbox::SearchEntryUpdated(text) => match self.active_view {
                View::Play => self.play_view.emit(play_view::Inbox::SearchStarted(text)),
                View::Browse => {}
            },
        }
    }
}
