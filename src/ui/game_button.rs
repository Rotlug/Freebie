use std::sync::Arc;

use adw::prelude::*;
use relm4::{gtk::gdk, prelude::*};

use crate::{game::Game, ui::bytes_to_texture};

pub struct GameButton {
    game: Arc<Game>,
    texture: gdk::Texture,
}

#[derive(Debug)]
pub enum Outbox {
    Clicked(Arc<Game>, gdk::Texture),
}

#[relm4::factory(pub async)]
impl AsyncFactoryComponent for GameButton {
    type Input = ();
    type Output = Outbox;
    type Init = Arc<Game>;
    type ParentWidget = gtk::FlowBox;
    type CommandOutput = ();

    view! {
        gtk::Box {
            set_orientation: gtk::Orientation::Vertical,
            set_valign: gtk::Align::Start,
            set_halign: gtk::Align::Center,

            adw::Clamp {
                set_maximum_size: 200,

                #[name = "cover_button"]
                gtk::Button {
                    set_overflow: gtk::Overflow::Hidden,

                    gtk::Box {
                        set_orientation: gtk::Orientation::Vertical,
                        set_hexpand: false,
                        set_vexpand: false,

                        #[name = "cover"]
                        gtk::Picture {
                            set_content_fit: gtk::ContentFit::Cover,
                            set_width_request: 200,
                            set_height_request: 300,
                        },

                        #[name = "title"]
                        gtk::Label {
                            set_label: &self.game.metadata.as_ref().unwrap().name,
                            set_ellipsize: gtk::pango::EllipsizeMode::End,
                            set_hexpand: true,
                            set_halign: gtk::Align::Start,
                            set_margin_top: 14,
                            set_margin_bottom: 14,
                            set_margin_start: 12,
                            set_margin_end: 12,
                        }
                    },

                    set_css_classes: &["card"]
                }
            }
        }
    }

    async fn init_model(
        init: Self::Init,
        _index: &DynamicIndex,
        _sender: AsyncFactorySender<Self>,
    ) -> Self {
        let metadata = init.metadata.as_ref().unwrap();
        let texture = if let Some(ref texture) = *metadata.cover.texture_cache.lock().await {
            texture.clone()
        } else {
            let bytes = metadata.cover.download().await.unwrap();
            let texture = bytes_to_texture(bytes).await.unwrap();
            *metadata.cover.texture_cache.lock().await = Some(texture.clone());
            texture
        };

        Self {
            game: init,
            texture,
        }
    }

    fn init_widgets(
        &mut self,
        _index: &DynamicIndex,
        root: Self::Root,
        _returned_widget: &<Self::ParentWidget as relm4::factory::FactoryView>::ReturnedWidget,
        sender: AsyncFactorySender<Self>,
    ) -> Self::Widgets {
        let widgets = view_output!();

        widgets.cover.set_paintable(Some(&self.texture));

        let outbox = sender.output_sender().clone();
        let video_game = self.game.clone();
        let texture = self.texture.clone();

        widgets.cover_button.connect_clicked(move |_| {
            outbox
                .send(Outbox::Clicked(video_game.clone(), texture.clone()))
                .unwrap();
        });

        widgets
    }
}
