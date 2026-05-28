use adw::prelude::*;
use relm4::prelude::*;

use crate::{
    igdb::{self},
    preferences::{Preferences, PreferencesInner},
    util::preferences_file,
};

pub struct PreferencesDialog {
    preferences: Preferences,
    root_window: adw::Window,
}

#[derive(Debug)]
pub enum Inbox {
    Present,
    Done,
}

#[relm4::component(pub async)]
impl AsyncComponent for PreferencesDialog {
    type Init = (adw::Window, Preferences);
    type Input = Inbox;
    type Output = ();
    type CommandOutput = ();

    view! {
        adw::PreferencesDialog {
            add = &adw::PreferencesPage {
                add = &adw::PreferencesGroup {
                    set_title: "Credentials",
                    set_description: Some("Used to fetch metadata like game covers from igdb"),

                     #[name = "client_id_entry"]
                     adw::EntryRow {
                        set_title: "Client ID",
                     },

                     #[name = "client_secret_entry"]
                     adw::PasswordEntryRow {
                        set_title: "Client Secret",
                     },
                }
            },

            connect_closed => Inbox::Done,
        }
    }

    async fn init(
        init: Self::Init,
        root: Self::Root,
        sender: AsyncComponentSender<Self>,
    ) -> AsyncComponentParts<Self> {
        let (root_window, preferences) = init;

        let model = Self {
            preferences,
            root_window,
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
                widgets
                    .client_id_entry
                    .set_text(&self.preferences.read().unwrap().credentials.client_id);
                widgets
                    .client_secret_entry
                    .set_text(&self.preferences.read().unwrap().credentials.client_secret);

                root.present(Some(&self.root_window));
            }
            Inbox::Done => {
                let new_preferenecs = PreferencesInner {
                    credentials: igdb::Credentials {
                        client_id: widgets.client_id_entry.text().to_string(),
                        client_secret: widgets.client_secret_entry.text().to_string(),
                    },
                };

                *self.preferences.write().unwrap() = new_preferenecs;

                let string = serde_json::to_string(&*self.preferences.read().unwrap()).unwrap();
                tokio::fs::write(preferences_file(), &string).await.unwrap();
            }
        }
        self.update_view(widgets, sender);
    }
}
