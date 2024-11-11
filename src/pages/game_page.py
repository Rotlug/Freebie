from gi.repository import Adw, Gtk, GLib, GdkPixbuf

from ..backend.game import Game
from PIL import Image, ImageFilter
from ..game_box import url_pixbuf
from ..game_manager import game_manager
from threading import Thread

@Gtk.Template(resource_path='/com/github/rotlug/Freebie/gtk/game_page.ui')
class GamePage(Adw.NavigationPage):
    __gtype_name__ = 'GamePage'
    blurred_background: Gtk.Picture = Gtk.Template.Child()
    window_title: Adw.WindowTitle = Gtk.Template.Child()

    game_title: Gtk.Label = Gtk.Template.Child()
    game_subtitle: Gtk.Label = Gtk.Template.Child()
    game_description: Gtk.Label = Gtk.Template.Child()

    cover: Gtk.Picture = Gtk.Template.Child()
    action_button: Gtk.Button = Gtk.Template.Child()

    def __init__(self):
        super().__init__()
        
        # Start the button update task in the background
        Thread(target=game_manager.update_button_task, )
    
    def set_game(self, game: Game):
        assert game.metadata != None

        self.blurred_background.set_pixbuf(get_blurred_pixbuf(game))
        self.window_title.set_title(game.name)
        
        self.game_title.set_label(game.name)

        rating = "No Rating Found" if game.metadata.rating == 0 else f"{game.metadata.rating}/100"

        self.game_subtitle.set_label(f"{rating} • {game.size}")
        self.game_description.set_label(game.metadata.description)
        
        self.cover.set_pixbuf(url_pixbuf(game))
        self.action_button.connect("clicked", get_game, game)

        game_manager.update_button(game, self.action_button)
    
def get_game(game: Game):
    print(f"GETTING GAME: {game.name}")
    Thread(target=game_manager.get_game, args=[game], daemon=True).start()

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