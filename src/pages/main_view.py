from gi.repository import Adw
from gi.repository import Gtk
from ..backend.game import Game

@Gtk.Template(resource_path='/com/github/rotlug/Freebie/gtk/main_view.ui')
class MainView(Adw.NavigationPage):
    __gtype_name__ = 'MainView'

    def __init__(self, **kwargs):
        super().__init__(**kwargs)