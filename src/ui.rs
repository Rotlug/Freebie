use glycin::Loader;
use image::imageops::FilterType;
use image::{ImageBuffer, Rgba};
use relm4::gtk::gdk::{self};

use adw::prelude::*;
use relm4::gtk::glib::Bytes;
use relm4::prelude::*;

pub mod action_button;
pub mod game_button;

pub mod add_game_dialog;
pub mod preferences_dialog;

pub mod browse_view;
pub mod play_view;

pub mod game_page;
pub mod main_page;
pub mod welcome_page;

/// Use `glycin` to convert a series of bytes into a paintable `gdk::Texture`
pub async fn bytes_to_texture(bytes: Vec<u8>) -> Result<gdk::Texture, glycin::ErrorCtx> {
    let image = Loader::new_vec(bytes).load().await?;

    let texture = image.next_frame().await?.texture();

    Ok(texture)
}

/// Downscales and applies a blur effect to `texture` using the CPU,
/// returning a new `gdk::Paintable` if successful.
fn blurred_paintable(texture: &gdk::Texture, radius: f64) -> Option<gdk::Paintable> {
    let target_width = 100;
    let target_height = 150;

    let mut texture_downloader = gdk::TextureDownloader::new(texture);
    texture_downloader.set_format(gdk::MemoryFormat::R8g8b8a8);
    let (raw_bytes, _stride) = texture_downloader.download_bytes();

    let width = texture.width().cast_unsigned();
    let height = texture.height().cast_unsigned();

    let img_buffer = ImageBuffer::<Rgba<u8>, _>::from_raw(width, height, raw_bytes.to_vec())?;

    let resized_img = image::imageops::resize(
        &img_buffer,
        target_width,
        target_height,
        FilterType::Lanczos3,
    );

    let blurred_img = image::imageops::blur(&resized_img, radius as f32);

    let final_bytes = Bytes::from(&blurred_img.into_raw());

    let final_texture = gdk::MemoryTexture::new(
        target_width.cast_signed(),
        target_height.cast_signed(),
        gdk::MemoryFormat::R8g8b8a8,
        &final_bytes,
        (target_width * 4) as usize,
    );

    Some(final_texture.upcast::<gdk::Paintable>())
}
