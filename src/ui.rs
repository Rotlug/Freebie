use glycin::Loader;
use relm4::gtk::gdk::{self};

use adw::prelude::*;
use relm4::prelude::*;

pub mod action_button;
pub mod browse_view;
pub mod game_button;
pub mod game_page;
pub mod main_page;

use relm4::gtk::graphene;

/// Use `glycin` to convert a series of bytes into a paintable `gdk::Texture`
pub async fn bytes_to_texture(bytes: Vec<u8>) -> Result<gdk::Texture, glycin::ErrorCtx> {
    let image = Loader::new_vec(bytes).load().await?;

    let texture = image.next_frame().await?.texture();

    Ok(texture)
}

/// Downscales and applies a blur effect to `texture`,
/// Returns a new `gdk::Paintable` if the transformation was successful. and `None` if it wasn't.
fn blurred_paintable(texture: &gdk::Texture, radius: f64) -> Option<gdk::Paintable> {
    let target_width = 100.0;
    let target_height = 150.0;

    let bounds = graphene::Rect::new(0.0, 0.0, target_width, target_height);
    let size = graphene::Size::new(target_width, target_height);

    let snapshot = gtk::Snapshot::new();

    snapshot.push_blur(radius);
    snapshot.append_texture(texture, &bounds);
    snapshot.pop();
    snapshot.to_paintable(Some(&size))
}
