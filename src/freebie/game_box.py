from gi.repository import Gtk, Gio, Adw
import urllib.request
from gi.repository.GdkPixbuf import Pixbuf

from .backend.utils import DATA_DIR

import os
from .backend.game import Game

pixbufs_cache_folder =  f"{DATA_DIR}/pixbufs"

def url_pixbuf(game: Game):
    if game.metadata is None: return
    
    pixbuf: Pixbuf | None
    file_name = f"{pixbufs_cache_folder}/{game.get_slug()}.png"

    if os.path.isfile(file_name):
        pixbuf = Pixbuf.new_from_file(file_name)
        return pixbuf

    url = "http://" + game.metadata.cover_url
    response = urllib.request.urlopen(url)
    input_stream = Gio.MemoryInputStream.new_from_data(response.read(), None)
    pixbuf = Pixbuf.new_from_stream(input_stream, None)
    
    assert type(pixbuf) == Pixbuf
    pixbuf.savev(file_name, "png")

    return pixbuf

def get_game_box(game: Game):
    return GameBox(game)

@Gtk.Template(resource_path='/com/github/rotlug/Freebie/gtk/game.ui')
class GameBox(Gtk.Box):
    __gtype_name__ = 'Game'

    title = Gtk.Template.Child()
    cover = Gtk.Template.Child()
    cover_button = Gtk.Template.Child()

    def __init__(self, game: Game, **kwargs):
        super().__init__(**kwargs)
        self.game = game

        self.title.set_label(game.name)
        self.cover.set_pixbuf(url_pixbuf(game))
    
    def connect_button(self, function):
        self.cover_button.connect("clicked", function, self.game)
