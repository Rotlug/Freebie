use adw::prelude::*;
use relm4::prelude::*;

mod error;
mod util;

mod game;
mod igdb;
mod ui;

struct App {}

#[relm4::component]
impl SimpleComponent for App {
    type Input = ();
    type Output = ();
    type Init = ();

    view! {
        #[name = "main_window"]
        adw::ApplicationWindow {
            set_default_size: (900, 500)
        }
    }

    fn init(
        init: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let model = App {};
        let widgets = view_output!();
        ComponentParts { model, widgets }
    }
}

fn main() {
    let app = RelmApp::new("land.lugasi.freebie");
    app.run::<App>(());
}
