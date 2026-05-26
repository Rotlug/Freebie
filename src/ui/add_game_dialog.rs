use adw::prelude::*;
use relm4::prelude::*;
use std::{
    path::PathBuf,
    sync::{Arc, Mutex},
    time::Duration,
};

use crate::{
    game::{self, Game},
    ui::main_page::open_executable_picker,
    util::slug::SlugExt,
};

pub struct AddGameDialog {
    game_name: String,
    exe_path: Option<PathBuf>,
    root_window: adw::Window,
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
    GameAdded(Game),
    Cancelled,
}

#[relm4::component(pub async)]
impl AsyncComponent for AddGameDialog {
    type Input = Inbox;
    type Output = Outbox;
    type Init = adw::Window;
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
        let model = Self {
            game_name: String::new(),
            exe_path: None,
            root_window: init,
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
                    _ = sender.output(Outbox::GameAdded(Game {
                        link: String::new(),
                        size: String::new(),
                        slug: self.game_name.slug(),
                        metadata: None,
                        state: Arc::new(Mutex::new(game::State::Installed {
                            path: exe_path.parent().unwrap().into(),
                            exe: exe_path,
                            time_played: Duration::default(),
                            is_open: false,
                        })),
                    }));
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
