use glycin::Loader;
use relm4::gtk::gdk::{self, prelude::TextureExt};

use adw::prelude::*;
use relm4::prelude::*;

pub mod browse_view;
pub mod game_button;
pub mod game_page;
pub mod main_page;

use relm4::gtk::graphene;

pub async fn bytes_to_texture(bytes: Vec<u8>) -> Result<gdk::Texture, glycin::ErrorCtx> {
    let image = Loader::new_vec(bytes).load().await?;

    let texture = image.next_frame().await?.texture();

    Ok(texture)
}

/// Replicates your Python pipeline: downscales the texture to 100x150,
/// and applies a Gaussian blur of radius 20.
fn blurred_paintable(texture: &gdk::Texture, radius: f64) -> Option<gdk::Paintable> {
    // 1. Force the target output size to 100x150
    let target_width = 100.0;
    let target_height = 150.0;

    // Set up the scaling and output canvas bounds
    let bounds = graphene::Rect::new(0.0, 0.0, target_width, target_height);
    let size = graphene::Size::new(target_width, target_height);

    let snapshot = gtk::Snapshot::new();

    // 2. Push your ImageFilter.GaussianBlur(20) state
    snapshot.push_blur(radius);

    // 3. Append the texture inside the 100x150 bounds.
    // This scales the texture down on the GPU automatically.
    snapshot.append_texture(texture, &bounds);

    snapshot.pop();

    // 4. Export the final 100x150 blurred paintable
    snapshot.to_paintable(Some(&size))
}
