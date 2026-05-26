use std::{path::PathBuf, sync::Arc};

use crate::{
    ActiveGames, TextureCache,
    game::Game,
    igdb::MetadataManager,
    settings::Settings,
    ui::{
        add_game_dialog::{self, AddGameDialog},
        browse_view::{self, BrowseView},
        play_view::{self, PlayView},
    },
    util::umu,
};
use adw::prelude::*;
use relm4::{
    RelmObjectExt,
    binding::{BoolBinding, ConnectBindingExt},
    prelude::*,
};

relm4::new_action_group!(AppActionGroup, "app");
relm4::new_stateless_action!(RunExeAction, AppActionGroup, "run_exe");
relm4::new_stateless_action!(AddGameAction, AppActionGroup, "add_game");

pub struct MainPage {
    root_window: adw::Window,

    // Views
    browse_view: AsyncController<BrowseView>,
    play_view: AsyncController<PlayView>,

    // Dialog to add games not from freebie
    add_game_dialog: AsyncController<AddGameDialog>,

    // Is the search bar currently visible
    search_visible: BoolBinding,

    // The currently visible view
    active_view: View,

    metadata: Arc<MetadataManager>,
    active_games: ActiveGames,
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
    RunExe(PathBuf),
    AddGame(Game),
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

                pack_end = &gtk::MenuButton {
                    set_icon_name: "open-menu-symbolic",
                    set_primary: true,
                    set_tooltip_text: Some("menu"),

                    set_menu_model: Some(&primary_menu)
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

        // Views
        let play_view = PlayView::builder()
            .launch((active_games.clone(), texture_cache.clone()))
            .forward(sender.output_sender(), |msg| match msg {
                play_view::Outbox::GameSelected(game, texture) => {
                    Outbox::GameSelected(game, texture)
                }
            });

        let add_game_dialog = AddGameDialog::builder()
            .launch(root_window.clone()) // Passes Init parameters (empty tuple in this case)
            .forward(sender.input_sender(), |output| match output {
                add_game_dialog::Outbox::GameAdded(game) => Inbox::AddGame(game),
                add_game_dialog::Outbox::Cancelled => Inbox::SearchBarEmpty,
            });

        // Search
        let search_visible = BoolBinding::new(false);

        // Menu
        relm4::menu! {
            primary_menu: {
                section! {
                    "Run Executable" => RunExeAction,
                    "Add Game" => AddGameAction
                }
            }
        }

        let mut app_group = relm4::actions::RelmActionGroup::<AppActionGroup>::new();

        let inbox = sender.input_sender().clone();
        let window = root_window.clone();
        let run_exe_action = relm4::actions::RelmAction::<RunExeAction>::new_stateless(move |_| {
            let inbox = inbox.clone();
            open_executable_picker(&window, move |path| {
                inbox.send(Inbox::RunExe(path)).unwrap();
            });
        });

        app_group.add_action(run_exe_action);

        let dialog = add_game_dialog.sender().clone();
        let add_game_action =
            relm4::actions::RelmAction::<AddGameAction>::new_stateless(move |_| {
                dialog.emit(add_game_dialog::Inbox::Present);
            });
        app_group.add_action(add_game_action);

        app_group.register_for_widget(&root_window);

        // Model
        let model = Self {
            root_window,
            browse_view,
            play_view,
            add_game_dialog,
            search_visible,
            active_view: View::Browse,
            metadata,
            active_games,
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
            Inbox::RunExe(path) => {
                tokio::spawn(async move {
                    _ = umu(&[&path.display().to_string()]).await;
                });
            }
            Inbox::AddGame(mut game) => {
                let slugs = [&game.slug];
                let Ok(metas) = self.metadata.get_games(&slugs).await else {
                    return;
                };

                for (meta_slug, meta) in metas {
                    if meta_slug == game.slug {
                        game.metadata = Some(meta);
                        self.active_games
                            .lock()
                            .unwrap()
                            .insert(meta_slug, Arc::new(game));

                        self.play_view.emit(play_view::Inbox::Update);
                        break;
                    }
                }
            }
        }
    }
}

/// Opens a file dialog restricted to executables (.exe, .lnk)
pub fn open_executable_picker<W, F>(parent_window: &W, on_success: F)
where
    W: IsA<gtk::Window>,
    F: FnOnce(PathBuf) + 'static,
{
    let file_picker = gtk::FileDialog::new();
    file_picker.set_title("Pick Executable");

    // Configure the file filters
    let filter = gtk::FileFilter::new();
    filter.set_name(Some("Executables (*.exe, *.lnk)"));
    filter.add_suffix("exe");
    filter.add_suffix("lnk");

    let filters = gtk::gio::ListStore::new::<gtk::FileFilter>();
    filters.append(&filter);
    file_picker.set_filters(Some(&filters));

    // Open the dialog
    file_picker.open(
        Some(parent_window),
        gtk::gio::Cancellable::NONE,
        move |file| {
            if let Ok(file) = file
                && let Some(path) = file.path()
            {
                on_success(path);
            }
        },
    );
}
