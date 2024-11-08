from gi.repository import Adw
from gi.repository import Gtk
from ..backend.game import Game

@Gtk.Template(resource_path='/com/github/rotlug/Freebie/gtk/window.ui')
class GameView(Gtk.Box):
    __gtype_name__ = 'GameView'

    def __init__(self):
        super().__init__()
        self.game: Game    
