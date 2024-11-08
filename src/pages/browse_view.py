from gi.repository import Adw
from gi.repository import Gtk
from ..game_manager import GameManager

from ..backend.igdb_api import IGDBApiWrapper
from ..game_box import GameBox

@Gtk.Template(resource_path='/com/github/rotlug/Freebie/gtk/browse_view.ui')
class BrowseView(Gtk.Box):
    __gtype_name__ = "BrowseView"

    library: Gtk.FlowBox = Gtk.Template.Child()

    def __init__(self, igdb: IGDBApiWrapper, game_manager: GameManager, **kwargs):
        super().__init__(**kwargs)

        games = game_manager.get_popular()

        for g in games:
            self.library.append(GameBox(g)) # type: ignore