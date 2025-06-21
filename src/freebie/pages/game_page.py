from typing import Union
from gi.repository import Adw, Gtk, GLib, GdkPixbuf
from gi.repository.Gio import InetSocketAddress

from ..backend.game import Game, InstalledGame
from PIL import Image, ImageFilter
from ..game_box import url_pixbuf
from ..game_manager import game_manager
from threading import Thread

from ..backend.igdb_api import igdb

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
    remove_button: Gtk.Button = Gtk.Template.Child()

    time_played: Gtk.Label = Gtk.Template.Child()

    def __init__(self, nav: Adw.NavigationView, window: Adw.ApplicationWindow):
        super().__init__()
        self.nav = nav
        self.window = window

        # Start the button update task in the background
        Thread(target=game_manager.update_button_task, args=[self, nav], daemon=True).start()
        self.game: Game

        self.action_button.connect("clicked", self.get_game)
        self.remove_button.connect("clicked", self.uninstall_game)
        
        game_manager.game_page = self

    def set_game(self, game: Game):
        if game.metadata is None:
            if isinstance(game, InstalledGame):
                game.metadata = igdb.search(game)
            else:
                return

        if (game.metadata is None):
            return
        
        # Time Played
        if (isinstance(game, InstalledGame)):
            self.time_played.set_visible(True)
            self.time_played.set_label(f"Time Played: {game_manager.format_duration(game.seconds_played)}")
        else:
            self.time_played.set_visible(False)

        self.game = game
        
        self.blurred_background.set_pixbuf(get_blurred_pixbuf(game))
        self.window_title.set_title(game.name)
        
        self.game_title.set_label(game.name)

        rating = "" if game.metadata.rating == 0 else f"{game.metadata.rating}/100"
        size = f" • {game.size}" if game.size != "" else game.size

        self.game_subtitle.set_label(rating + size)
        
        if game.metadata.description != None:
            self.game_description.set_label(game.metadata.description)
        
        self.cover.set_pixbuf(url_pixbuf(game))
        
        # Update button state when entering the page
        game_manager.update_button(game, self.action_button)

        self.remove_button.set_visible(game_manager.is_installed(game))
        
    def get_game(self, widget):
        self.action_button.set_sensitive(False)
        target = game_manager.get_button_target(self.game)
        target(self.game)
    
    def uninstall_game(self, widget):
        dialog = Adw.AlertDialog(
            heading="Delete Game?",
            body=f"Are you sure you want to delete <b>{self.game.name}</b>? This action cannot be undone!",
            body_use_markup=True,
            default_response="cancel",
        )

        dialog.add_response("cancel", "Cancel")
        dialog.add_response("delete", "Delete")
        dialog.set_response_appearance("delete", Adw.ResponseAppearance.DESTRUCTIVE)
        
        dialog.connect("response", self.on_delete_dialog_response)
        dialog.present(self.window)

    def on_delete_dialog_response(self, dialog, response):
        if response == "delete":
            game_manager.uninstall(self.game)
            self.set_game(self.game)
            self.nav.pop_to_tag("main")
    
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
