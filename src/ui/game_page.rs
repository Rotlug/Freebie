//! The game page is the page where the user can look at the games metadata and install it.

use adw::prelude::*;
use relm4::{
    actions::{RelmAction, RelmActionGroup},
    prelude::*,
};
use std::sync::Arc;

use crate::{
    ActiveGames,
    game::Game,
    ui::{
        action_button::{self, ActionButton},
        blurred_paintable,
    },
};

relm4::new_action_group!(GameActionGroup, "game");
relm4::new_stateless_action!(
    MakeDesktopShortcutAction,
    GameActionGroup,
    "make_desktop_shortcut"
);

#[derive(Debug)]
pub enum Inbox {
    ChangeGame(Arc<Game>, gtk::gdk::Texture),
    Update(Arc<Game>),
    ActionButtonClicked,
    DeleteButtonClicked,
    MakeDesktopShortcut,
}

#[derive(Debug)]
pub enum Outbox {
    GameUninstalled(Arc<Game>),
}

pub struct GamePage {
    game: Option<Arc<Game>>,
    texture: Option<gtk::gdk::Texture>,
    action_button: AsyncController<ActionButton>,
    active_games: ActiveGames,
    root_window: adw::Window,
}

#[relm4::component(pub async)]
impl AsyncComponent for GamePage {
    type Input = Inbox;
    type Output = Outbox;
    type Init = (ActiveGames, adw::Window);
    type CommandOutput = ();

    view! {
        adw::ToastOverlay {
            gtk::Overlay {
                #[name = "blurred_background"]
                #[wrap(Some)]
                set_child = &gtk::Picture {
                    set_opacity: 0.25,
                    set_content_fit: gtk::ContentFit::Fill
                },

                add_overlay = &adw::ToolbarView {
                    add_top_bar = &adw::HeaderBar {
                        #[wrap(Some)]
                        set_title_widget = &adw::WindowTitle {
                            #[watch]
                            set_title: model.game.as_ref().map_or("Title", |game| {
                                &game.metadata.as_ref().unwrap().name
                            })
                        }
                    },

                    #[wrap(Some)]
                    set_content = &gtk::ScrolledWindow {
                        gtk::Box {
                            set_margin_all: 24,

                            adw::Clamp {
                                set_maximum_size: 200,

                                #[name = "cover"]
                                gtk::Picture {
                                    set_halign: gtk::Align::End,
                                    set_valign: gtk::Align::Start,

                                    set_content_fit: gtk::ContentFit::Cover,
                                    set_width_request: 200,
                                    set_height_request: 300,

                                    set_css_classes: &["card"]
                                }
                            },

                            gtk::Box {
                                set_orientation: gtk::Orientation::Vertical,
                                set_margin_start: 10,
                                set_hexpand: true,

                                gtk::Label {
                                    set_wrap: true,
                                    set_wrap_mode: gtk::pango::WrapMode::Word,
                                    set_halign: gtk::Align::Start,

                                    #[watch]
                                    set_label: model.game.as_ref().map_or("Title", |game| {
                                        &game.metadata.as_ref().unwrap().name
                                    }),

                                    set_css_classes: &["title-1"]
                                },

                                gtk::Label {
                                    set_wrap: true,
                                    set_wrap_mode: gtk::pango::WrapMode::Word,
                                    set_halign: gtk::Align::Start,

                                    #[watch]
                                    set_label: &model.subtitle(),

                                    set_css_classes: &["heading"],
                                },

                                gtk::Label {
                                    set_wrap: true,
                                    set_wrap_mode: gtk::pango::WrapMode::Word,
                                    set_halign: gtk::Align::Start,
                                    set_margin_top: 10,

                                    #[watch]
                                    set_label: &model.description(),

                                    set_css_classes: &["body"],
                                },

                                gtk::Box {
                                    set_orientation: gtk::Orientation::Horizontal,
                                    set_margin_top: 10,
                                    set_spacing: 10,

                                    append = model.action_button.widget(),

                                    #[name = "delete_button"]
                                    gtk::Button {
                                        set_label: "Delete",
                                        set_css_classes: &["destructive-action"],

                                        set_visible: false,
                                        connect_clicked => Inbox::DeleteButtonClicked,
                                    },

                                    #[name = "menu_button"]
                                    gtk::MenuButton {
                                        set_menu_model: Some(&primary_menu),
                                        set_icon_name: "view-more-symbolic",
                                    }
                                }
                            }
                        }
                    }
                },
            },
        }
    }

    async fn init(
        init: Self::Init,
        root: Self::Root,
        sender: AsyncComponentSender<Self>,
    ) -> AsyncComponentParts<Self> {
        let (active_games, root_window) = init;
        let action_button =
            ActionButton::builder()
                .launch(())
                .forward(sender.input_sender(), |msg| match msg {
                    action_button::Outbox::Clicked => Inbox::ActionButtonClicked,
                    action_button::Outbox::Update(game) => Inbox::Update(game),
                });

        // Menu
        relm4::menu! {
            primary_menu: {
                section! {
                    "Make desktop shortcut" => MakeDesktopShortcutAction,
                }
            }
        }

        let mut group = RelmActionGroup::<GameActionGroup>::new();

        let sender_ = sender.clone();
        let make_desktop_shortcut_action =
            RelmAction::<MakeDesktopShortcutAction>::new_stateless(move |_| {
                sender_.input(Inbox::MakeDesktopShortcut);
            });
        group.add_action(make_desktop_shortcut_action);

        group.register_for_widget(&root);

        let model = Self {
            active_games,
            root_window,
            game: None,
            texture: None,
            action_button,
        };
        let widgets = view_output!();

        AsyncComponentParts { model, widgets }
    }

    async fn update_with_view(
        &mut self,
        widgets: &mut Self::Widgets,
        message: Self::Input,
        sender: AsyncComponentSender<Self>,
        _root: &Self::Root,
    ) {
        match message {
            Inbox::Update(game) => {
                widgets.delete_button.set_visible(game.installed());
                widgets.menu_button.set_visible(game.installed());
                self.action_button
                    .emit(action_button::Inbox::Update(game.clone()));
            }
            Inbox::ChangeGame(game, texture) => {
                sender.input(Inbox::Update(game.clone()));

                self.game = Some(game.clone());
                self.texture = Some(texture.clone());

                // set cover pic
                widgets.cover.set_paintable(Some(&texture));
                // set the background to a blurred version of the cover
                let blurred = blurred_paintable(&texture, 10.0).unwrap();
                widgets.blurred_background.set_paintable(Some(&blurred));
            }
            Inbox::ActionButtonClicked => {
                if let Some(game) = self.game.clone() {
                    self.active_games
                        .lock()
                        .unwrap()
                        .insert(game.slug.clone(), game.clone());

                    self.action_button
                        .emit(action_button::Inbox::GameAction(game));
                }
            }
            Inbox::DeleteButtonClicked => {
                if let Some(ref game) = self.game {
                    let dialog = adw::AlertDialog::builder()
                        .heading("Are you sure?")
                        .body(format!("Are you sure you want to delete <b>{}</b>? This removes all game files!", game.metadata.as_ref().unwrap().name))
                        .body_use_markup(true)
                        .default_response("cancel").build();

                    dialog.add_response("delete", "Delete");
                    dialog.set_response_appearance("delete", adw::ResponseAppearance::Destructive);
                    dialog.add_response("cancel", "Cancel");

                    let selected = dialog.choose_future(Some(&self.root_window)).await;

                    if selected == "delete" {
                        _ = game.uninstall().await;
                        sender.input(Inbox::Update(game.clone()));
                        let _ = sender.output(Outbox::GameUninstalled(game.clone()));
                    }
                }
            }
            Inbox::MakeDesktopShortcut => {
                if let Some(ref game) = self.game {
                    _ = game.make_shortcut().await;
                }
            }
        }

        self.update_view(widgets, sender);
    }
}

/// Functions to display the games info
impl GamePage {
    fn description(&self) -> &str {
        self.game
            .as_ref()
            .and_then(|game| game.metadata.as_ref())
            .and_then(|metadata| metadata.description.as_ref())
            .map_or_else(|| "No description available", |metadata| metadata.as_str())
    }

    fn subtitle(&self) -> String {
        let Some(game) = &self.game else {
            return String::new();
        };

        let rating = game.metadata.as_ref().unwrap().rating.map(|rating| {
            let rating = (rating as i32).to_string();
            format!("{rating}/100")
        });

        let size = &game.size;

        match rating {
            Some(rating) => format!("{rating} • {size}"),
            None => size.into(),
        }
    }
}
