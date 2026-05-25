//! The "Action Button" is the button for download, playing, etc.

use std::{sync::Arc, time::Duration};

use adw::prelude::*;
use relm4::prelude::*;

use crate::{
    error::{DownloadError, InstallError},
    game::{self, Game},
    util::downloads,
};

#[derive(Debug)]
pub enum Inbox {
    GameAction(Arc<Game>),
    Update(Arc<Game>),
}

#[derive(Debug)]
pub enum Outbox {
    Clicked,
}

#[derive(Debug)]
pub enum Command {
    DownloadFail(DownloadError, tokio::task::JoinHandle<()>),
    InstallError(InstallError, tokio::task::JoinHandle<()>),
    GameInstalled(Arc<Game>),
    GameClosed,
}

pub struct ActionButton {
    session: Arc<librqbit::Session>,
}

#[relm4::component(pub async)]
impl AsyncComponent for ActionButton {
    type Input = Inbox;
    type Output = Outbox;
    type Init = ();
    type CommandOutput = Command;

    view! {
        gtk::Button {
            set_label: "Get",
            set_css_classes: &["suggested-action"]
        }
    }

    async fn init(
        _init: Self::Init,
        root: Self::Root,
        sender: AsyncComponentSender<Self>,
    ) -> AsyncComponentParts<Self> {
        let session = librqbit::Session::new(downloads()).await.unwrap();
        let model = Self { session };
        let widgets = view_output!();

        let outbox = sender.output_sender().clone();
        root.connect_clicked(move |_| {
            outbox.clone().send(Outbox::Clicked).unwrap();
        });

        AsyncComponentParts { model, widgets }
    }

    async fn update(
        &mut self,
        message: Self::Input,
        sender: AsyncComponentSender<Self>,
        root: &Self::Root,
    ) {
        match message {
            Inbox::GameAction(game) => {
                let state = &*game.state.lock().unwrap();

                match state {
                    game::State::Uninstalled => {
                        sender.input(Inbox::Update(game.clone()));
                        let game = game.clone();
                        let inbox = sender.input_sender().clone();
                        let session = self.session.clone();

                        sender.oneshot_command(async move {
                            // Background task updating the action button ui
                            let game_tracker = game.clone();
                            let updater_inbox = inbox.clone();
                            let updater = tokio::spawn(async move {
                                loop {
                                    updater_inbox
                                        .send(Inbox::Update(game_tracker.clone()))
                                        .unwrap();
                                    tokio::time::sleep(Duration::from_secs(1)).await;
                                }
                            });

                            let download = match game.download(&session).await {
                                Ok(download) => download,
                                Err(err) => return Command::DownloadFail(err, updater),
                            };

                            if let Err(err) = game.install(download).await {
                                return Command::InstallError(err, updater);
                            }

                            updater.abort();
                            inbox.send(Inbox::Update(game.clone())).unwrap();
                            Command::GameInstalled(game)
                        });
                    }
                    game::State::Installed { .. } => {
                        let game_cmd = game.clone();
                        let inbox = sender.input_sender().clone();

                        root.set_label("Playing");
                        root.set_sensitive(false);
                        root.set_css_classes(&[]);
                        sender.oneshot_command(async move {
                            _ = game_cmd.play().await;
                            inbox.send(Inbox::Update(game_cmd)).unwrap();
                            Command::GameClosed
                        });
                    }
                    _ => panic!("ActionButton should not be pressable"),
                }
            }
            Inbox::Update(game) => {
                let sensitive = Self::sensitive(&game);
                root.set_css_classes(if sensitive {
                    &["suggested-action"]
                } else {
                    &[]
                });
                root.set_sensitive(sensitive);
                root.set_label(&Self::label(&game));
            }
        }
    }

    async fn update_cmd(
        &mut self,
        message: Self::CommandOutput,
        _sender: AsyncComponentSender<Self>,
        _root: &Self::Root,
    ) {
        match message {
            Command::DownloadFail(err, updater) => {
                dbg!(err);
                updater.abort();
            }
            Command::InstallError(err, updater) => {
                dbg!(err);
                updater.abort();
            }
            Command::GameClosed => {}
            Command::GameInstalled(game) => {
                let state = &*game.state.lock().unwrap();
                if let game::State::Installed { exe, .. } = state {
                    println!("Game installed successfuly!");
                    println!("Desktop shortcut found in: {}", exe.display());
                }
            }
        }
    }
}

impl ActionButton {
    fn sensitive(game: &Game) -> bool {
        let state = &*game.state.lock().unwrap();

        matches!(
            state,
            game::State::Uninstalled | game::State::Installed { .. }
        )
    }

    fn label(game: &Game) -> String {
        let state = &*game.state.lock().unwrap();

        match state {
            game::State::Uninstalled => "Get".into(),
            game::State::Preparing => "Preparing for download".into(),
            game::State::Downloading(percentage) => format!("Downloading... ({percentage})"),
            game::State::Installing => "Installing...".into(),
            game::State::Installed { .. } => "Play".into(),
        }
    }
}
