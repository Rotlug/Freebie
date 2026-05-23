use adw::prelude::*;
use relm4::prelude::*;
use std::sync::Arc;

use crate::{game::Game, ui::blurred_paintable};

#[derive(Debug)]
pub enum Inbox {
    NewGame(Arc<Game>, gtk::gdk::Texture),
}

pub struct GamePage {
    game: Option<Arc<Game>>,
    texture: Option<gtk::gdk::Texture>,
}

#[relm4::component(pub async)]
impl AsyncComponent for GamePage {
    type Input = Inbox;
    type Output = ();
    type Init = ();
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

                                    set_css_classes: &["document"],
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
        let model = Self {
            game: None,
            texture: None,
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
            Inbox::NewGame(game, texture) => {
                self.game = Some(game);
                self.texture = Some(texture.clone());

                widgets.cover.set_paintable(Some(&texture));
                let blurred = blurred_paintable(&texture, 20.0).unwrap();
                widgets.blurred_background.set_paintable(Some(&blurred));
            }
        }

        self.update_view(widgets, sender);
    }
}

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
