use std::{path::PathBuf, sync::Arc};

use crate::{
    ActiveGames, TextureCache,
    game::Game,
    igdb::MetadataManager,
    preferences::Preferences,
    ui::{
        add_game_dialog::{self, AddGameDialog, open_executable_picker},
        browse_view::{self, BrowseView},
        play_view::{self, PlayView},
        preferences_dialog::{self, PreferencesDialog},
    },
    util::umu,
};
use adw::prelude::*;
use relm4::{
    RelmObjectExt,
    actions::{AccelsPlus, RelmAction, RelmActionGroup},
    binding::{BoolBinding, ConnectBindingExt},
    prelude::*,
};

relm4::new_action_group!(AppActionGroup, "app");
relm4::new_stateless_action!(RunExeAction, AppActionGroup, "run_exe");
relm4::new_stateless_action!(AddGameAction, AppActionGroup, "add_game");
relm4::new_stateless_action!(
    KeyboardShortcutsAction,
    AppActionGroup,
    "keyboard_shortcuts"
);
relm4::new_stateless_action!(PreferencesAction, AppActionGroup, "preferences");
relm4::new_stateless_action!(AboutAction, AppActionGroup, "about");

pub struct MainPage {
    root_window: adw::Window,

    // Views
    browse_view: AsyncController<BrowseView>,
    play_view: AsyncController<PlayView>,

    // Is the search bar currently visible
    search_visible: BoolBinding,

    add_game_dialog: AsyncController<AddGameDialog>,
    preferences_dialog: AsyncController<PreferencesDialog>,

    // The currently visible view
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
    SearchUpdated(String),
    /// fires every time the search entry text changes, disregarding the `search_delay`.
    SearchEntryUpdated(String),
    GameUninstalled,
    ShowKeyboardShortcuts,
    ShowAboutDialog,
    ViewChanged(View),
    /// "Run" Executable action happened
    RunExe(PathBuf),
    /// Game installed from the "Add game" Dialog. we need to update the play view.
    GameAdded,
}

#[relm4::component(pub async)]
impl AsyncComponent for MainPage {
    type Input = Inbox;
    type Output = Outbox;
    type Init = (adw::Window, ActiveGames, TextureCache, Preferences);
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
                },

                connect_visible_child_notify[sender] => move |stack| {
                    if let Some(visible_child) = stack.visible_child_name() {
                        match visible_child.as_str() {
                            "browse" => sender.input(Inbox::ViewChanged(View::Browse)),
                            "play" => sender.input(Inbox::ViewChanged(View::Play)),
                            _ => {}
                        }
                    }
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
                        set_search_delay: 300,

                        connect_search_changed[sender] => move |entry| {
                            sender.input(Inbox::SearchUpdated(entry.text().to_string()));
                        },

                        connect_search_changed[sender] => move |entry| {
                            sender.input(Inbox::SearchEntryUpdated(entry.text().to_string()));
                        }
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
        let (root_window, active_games, texture_cache, preferences) = init;
        let metadata = Arc::new(MetadataManager::new(preferences.clone()));

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
            .launch((root_window.clone(), active_games.clone(), metadata.clone())) // Passes Init parameters (empty tuple in this case)
            .forward(sender.input_sender(), |output| match output {
                add_game_dialog::Outbox::GameAdded => Inbox::GameAdded,
                add_game_dialog::Outbox::Cancelled => Inbox::SearchUpdated(String::new()),
            });

        let preferences_dialog = PreferencesDialog::builder()
            .launch((root_window.clone(), preferences.clone()))
            .detach();

        // Search
        let search_visible = BoolBinding::new(false);

        // Menu
        relm4::menu! {
            primary_menu: {
                section! {
                    "Run Executable" => RunExeAction,
                    "Add Game" => AddGameAction,
                    "Preferences" => PreferencesAction,
                    "Keyboard Shortcuts" => KeyboardShortcutsAction,
                    "About Freebie" => AboutAction
                }
            }
        }

        let app = relm4::main_adw_application();
        app.set_accelerators_for_action::<RunExeAction>(&["<primary>E"]);
        app.set_accelerators_for_action::<AddGameAction>(&["<primary>P"]);
        app.set_accelerators_for_action::<KeyboardShortcutsAction>(&["<primary>question"]);

        let mut group = RelmActionGroup::<AppActionGroup>::new();

        let sender_ = sender.clone();
        let window = root_window.clone();
        let run_exe_action = RelmAction::<RunExeAction>::new_stateless(move |_| {
            let sender_ = sender_.clone();
            open_executable_picker(&window, move |path| {
                sender_.input(Inbox::RunExe(path));
            });
        });
        group.add_action(run_exe_action);

        let dialog = add_game_dialog.sender().clone();
        let add_game_action = RelmAction::<AddGameAction>::new_stateless(move |_| {
            dialog.emit(add_game_dialog::Inbox::Present);
        });
        group.add_action(add_game_action);

        let sender_ = sender.clone();
        let keyboard_shortcuts_action =
            RelmAction::<KeyboardShortcutsAction>::new_stateless(move |_| {
                sender_.input(Inbox::ShowKeyboardShortcuts);
            });
        group.add_action(keyboard_shortcuts_action);

        let dialog = preferences_dialog.sender().clone();
        let add_game_action = RelmAction::<PreferencesAction>::new_stateless(move |_| {
            dialog.emit(preferences_dialog::Inbox::Present);
        });
        group.add_action(add_game_action);

        let sender_ = sender.clone();
        let about_action = RelmAction::<AboutAction>::new_stateless(move |_| {
            sender_.input(Inbox::ShowAboutDialog);
        });
        group.add_action(about_action);

        group.register_for_widget(&root_window);

        // Final
        let model = Self {
            root_window,
            browse_view,
            play_view,
            search_visible,
            add_game_dialog,
            preferences_dialog,
            active_view: View::Browse,
        };

        let widgets = view_output!();
        widgets.search_bar.connect_entry(&widgets.search_entry);

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
                    View::Browse => self.browse_view.emit(browse_view::Inbox::ShowPopular),
                }
                self.active_view = view;
            }
            Inbox::SearchUpdated(query) => match self.active_view {
                View::Browse => self.browse_view.emit(match query.len() {
                    0 => browse_view::Inbox::ShowPopular,
                    3.. => browse_view::Inbox::NewQuery(query),
                    _ => return,
                }),
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
            Inbox::GameUninstalled => match self.active_view {
                View::Browse => {}
                View::Play => {
                    self.play_view.emit(play_view::Inbox::Update);
                }
            },
            Inbox::GameAdded => {
                self.play_view.emit(play_view::Inbox::Update);
            }
            Inbox::ShowKeyboardShortcuts => {
                let shortcuts_dialog = adw::ShortcutsDialog::new();

                let general_section = adw::ShortcutsSection::new(Some("General"));

                let add_game_shortcut = adw::ShortcutsItem::new("Add Game", "<primary>p");
                let run_exe_shortcut = adw::ShortcutsItem::new("Run executable", "<primary>e");

                general_section.add(add_game_shortcut);
                general_section.add(run_exe_shortcut);

                shortcuts_dialog.add(general_section);
                shortcuts_dialog.present(Some(&self.root_window));
            }
            Inbox::ShowAboutDialog => {
                let about_dialog = adw::AboutDialog::builder()
                    .application_name("Freebie")
                    .developer_name("Rotlug")
                    .developers(["Rotlug"])
                    .version("2.0")
                    .build();

                about_dialog.present(Some(&self.root_window));
            }
        }
    }
}
