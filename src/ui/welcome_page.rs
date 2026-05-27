use std::sync::{Arc, RwLock};

use adw::prelude::*;
use relm4::prelude::*;

use crate::{
    igdb,
    preferences::{Preferences, PreferencesInner},
};

#[derive(Debug)]
pub enum Inbox {
    Start,
    Done,
}

#[derive(Debug)]
pub enum Outbox {
    Done(Preferences),
}

pub struct WelcomePage {}

#[relm4::component(pub async)]
impl AsyncComponent for WelcomePage {
    type Init = ();
    type Input = Inbox;
    type Output = Outbox;
    type CommandOutput = ();

    view! {
        adw::ToolbarView {
            add_top_bar = &adw::HeaderBar {
                set_show_title: false,
            },

            gtk::Box {
                set_orientation: gtk::Orientation::Vertical,

                #[name = "carousel"]
                adw::Carousel {
                    set_interactive: false,
                    set_vexpand: true,
                    set_spacing: 100,

                    #[name = "start_page"]
                    gtk::Box {
                        set_valign: gtk::Align::Center,
                        set_halign: gtk::Align::Center,
                        set_orientation: gtk::Orientation::Vertical,
                        set_spacing: 12,

                        gtk::Label {
                            set_label: "Welcome to Freebie",
                            set_css_classes: &["title-1"]
                        },

                        gtk::Label {
                            set_label: "Lets get your credentials before we get started",
                            set_css_classes: &["body"]
                        },

                        #[name = "start_button"]
                        gtk::Button {
                            set_label: "Ok",
                            set_halign: gtk::Align::Center,
                            set_css_classes: &["pill", "suggested-action"],
                            connect_clicked => Inbox::Start
                        }
                    },

                    #[name = "credentials_page"]
                    gtk::Box {
                        set_valign: gtk::Align::Center,
                        set_halign: gtk::Align::Center,
                        set_orientation: gtk::Orientation::Vertical,
                        set_spacing: 12,

                        gtk::Label {
                            set_label: "Insert your credentials",
                            set_css_classes: &["title-1"]
                        },

                        gtk::ListBox {
                            set_css_classes: &["boxed-list"],

                            #[name = "client_id_entry"]
                            adw::EntryRow {
                               set_title: "Client ID",
                            },

                            #[name = "client_secret_entry"]
                            adw::PasswordEntryRow {
                               set_title: "Client Secret",
                            },
                        },

                        #[name = "done_button"]
                        gtk::Button {
                            set_label: "Done",
                            set_halign: gtk::Align::Center,
                            set_css_classes: &["pill", "suggested-action"],
                            connect_clicked => Inbox::Done,
                        }
                    },
                },

                adw::CarouselIndicatorLines {
                    set_margin_bottom: 5,
                    set_margin_top: 5,
                    set_carousel: Some(&carousel)
                }
            },
        }
    }

    async fn init(
        _init: Self::Init,
        root: Self::Root,
        sender: AsyncComponentSender<Self>,
    ) -> AsyncComponentParts<Self> {
        let model = Self {};
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
            Inbox::Start => {
                widgets.carousel.scroll_to(&widgets.credentials_page, true);
            }
            Inbox::Done => {
                let client_id = widgets.client_id_entry.text().to_string();
                let client_secret = widgets.client_secret_entry.text().to_string();

                _ = sender.output(Outbox::Done(Arc::new(RwLock::new(PreferencesInner {
                    credentials: igdb::Credentials {
                        client_id,
                        client_secret,
                    },
                }))));
            }
        }
    }
}
