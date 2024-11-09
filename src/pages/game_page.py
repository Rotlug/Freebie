from gi.repository import Adw, Gtk, GLib, GdkPixbuf

from ..backend.game import Game
from PIL import Image, ImageFilter
from ..game_box import url_pixbuf

@Gtk.Template(resource_path='/com/github/rotlug/Freebie/gtk/game_page.ui')
class GamePage(Adw.NavigationPage):
    __gtype_name__ = 'GamePage'
    blurred_background: Gtk.Picture = Gtk.Template.Child()

    def __init__(self):
        super().__init__()
    
    def set_game(self, game: Game):
        self.blurred_background.set_pixbuf(get_blurred_pixbuf(game))

def get_blurred_pixbuf(game: Game):
    pixbuf = url_pixbuf(game)
    assert pixbuf != None
    image = pixbuf2image(pixbuf)
    image = (
        image.convert("RGB")
        .resize((100, 150))
        .filter(ImageFilter.GaussianBlur(20))
    )

    return image2pixbuf(image)

def image2pixbuf(im: Image.Image):
    """Convert Pillow image to GdkPixbuf"""
    data = im.tobytes()
    w, h = im.size
    data = GLib.Bytes.new(data) # type: ignore
    pix = GdkPixbuf.Pixbuf.new_from_bytes(data, GdkPixbuf.Colorspace.RGB, False, 8, w, h, w * 3)
    return pix

def pixbuf2image(pix: GdkPixbuf.Pixbuf):
    """Convert gdkpixbuf to PIL image"""
    data = pix.get_pixels()
    w = pix.props.width
    h = pix.props.height
    stride = pix.props.rowstride
    mode = "RGB"
    if pix.props.has_alpha == True:
        mode = "RGBA"
    im = Image.frombytes(mode, (w, h), data, "raw", mode, stride)
    return im