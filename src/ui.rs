use glycin::Loader;
use relm4::gtk::gdk;

pub async fn bytes_to_texture(bytes: Vec<u8>) -> Result<gdk::Texture, glycin::ErrorCtx> {
    let image = Loader::new_vec(bytes).load().await?;

    let texture = image.next_frame().await?.texture();

    Ok(texture)
}
