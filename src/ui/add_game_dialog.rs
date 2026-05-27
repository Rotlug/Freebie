use adw::prelude::*;
use relm4::prelude::*;
use std::{
    path::PathBuf,
    sync::{Arc, RwLock},
    time::Duration,
};

use crate::{
    ActiveGames,
    game::{self, Game},
    igdb::MetadataManager,
    util::slug::SlugExt,
};

pub struct AddGameDialog {
    game_name: String,
    exe_path: Option<PathBuf>,
    root_window: adw::Window,
    active_games: ActiveGames,
    metadata: Arc<MetadataManager>,
}

#[derive(Debug)]
pub enum Inbox {
    Present,
    NameChanged(String),
    PickExe,
    ExeSelected(PathBuf),
    Cancel,
    Add,
}

#[derive(Debug)]
pub enum Outbox {
    GameAdded,
    Cancelled,
}

#[relm4::component(pub async)]
impl AsyncComponent for AddGameDialog {
    type Input = Inbox;
    type Output = Outbox;
    type Init = (adw::Window, ActiveGames, Arc<MetadataManager>);
    type CommandOutput = ();

    view! {
        adw::Dialog {
            set_title: "Add Game",
            set_content_width: 500,

            // Automatically close the dialog when the user clicks out or hits Escape
            set_can_close: true,

            #[wrap(Some)]
            set_child = &adw::ToolbarView {
                add_top_bar = &adw::HeaderBar {
                    set_show_start_title_buttons: false,
                    set_show_end_title_buttons: false,

                    #[name = "cancel_button"]
                    pack_start = &gtk::Button {
                        set_label: "Cancel",
                        connect_clicked => Inbox::Cancel,
                    },

                    #[name = "add_button"]
                    pack_end = &gtk::Button {
                        set_label: "Add",
                        add_css_class: "suggested-action",

                        #[watch]
                        set_sensitive: model.is_sensitive(),

                        connect_clicked => Inbox::Add,
                    },
                },

                #[wrap(Some)]
                set_content = &adw::PreferencesPage {
                    add = &adw::PreferencesGroup {
                        #[name = "game_name_row"]
                        add = &adw::EntryRow {
                            set_title: "Game Name",
                            connect_changed[sender] => move |row| {
                                sender.input(Inbox::NameChanged(row.text().to_string()));
                            }
                        },

                        #[name = "select_exe_row"]
                        add = &adw::ActionRow {
                            set_title: "Executable",

                            #[watch]
                            set_subtitle: &model.exe_subtitle(),

                            add_suffix = &gtk::Button {
                                set_valign: gtk::Align::Center,
                                set_icon_name: "document-open-symbolic",
                                set_tooltip_text: Some("Select File"),
                                add_css_class: "flat",

                                connect_clicked => Inbox::PickExe,
                            }
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
        let (root_window, active_games, metadata) = init;
        let model = Self {
            game_name: String::new(),
            exe_path: None,
            root_window,
            active_games,
            metadata,
        };

        let widgets = view_output!();

        AsyncComponentParts { model, widgets }
    }

    async fn update_with_view(
        &mut self,
        widgets: &mut Self::Widgets,
        message: Self::Input,
        sender: AsyncComponentSender<Self>,
        root: &Self::Root,
    ) {
        match message {
            Inbox::Present => {
                self.reset();
                widgets.game_name_row.set_text("");
                widgets.select_exe_row.set_subtitle(&self.exe_subtitle());

                root.present(Some(&self.root_window));
            }
            Inbox::NameChanged(name) => {
                self.game_name = name;
            }
            Inbox::PickExe => {
                let inbox = sender.input_sender().clone();

                open_executable_picker(&self.root_window, move |path| {
                    _ = inbox.send(Inbox::ExeSelected(path));
                });
            }
            Inbox::ExeSelected(path) => {
                self.exe_path = Some(path);
            }
            Inbox::Cancel => {
                root.close();
                _ = sender.output(Outbox::Cancelled);
            }
            Inbox::Add => {
                if let Some(exe_path) = self.exe_path.take() {
                    root.close();
                    let mut game = Game {
                        link: String::new(),
                        size: String::new(),
                        slug: self.game_name.slug(),
                        metadata: None,
                        state: Arc::new(RwLock::new(game::State::Installed {
                            path: exe_path.parent().unwrap().into(),
                            exe: exe_path,
                            time_played: Duration::default(),
                            is_open: false,
                        })),
                    };

                    let Ok(metas) = self.metadata.get_games(&[&game.slug]).await else {
                        return;
                    };

                    for (meta_slug, meta) in metas {
                        if meta_slug == game.slug {
                            game.metadata = Some(meta);
                            let game = Arc::new(game);
                            self.active_games
                                .lock()
                                .unwrap()
                                .insert(game.slug.clone(), game.clone());
                            _ = sender.output(Outbox::GameAdded);
                            return;
                        }
                    }
                }
            }
        }

        self.update_view(widgets, sender);
    }
}

impl AddGameDialog {
    fn exe_subtitle(&self) -> String {
        match &self.exe_path {
            Some(path) => path.display().to_string(),
            None => String::from("Select the .exe file of the game"),
        }
    }

    fn is_sensitive(&self) -> bool {
        self.exe_path.is_some() && !self.game_name.is_empty()
    }

    fn reset(&mut self) {
        self.exe_path = None;
        self.game_name = String::new();
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
