from threading import Thread
from gi.repository import Adw, Gtk, GLib
from ..pages.game_page import GamePage
from ..backend.game import Game

from ..game_manager import game_manager
from ..backend.igdb_api import igdb

from ..game_box import GameBox

@Gtk.Template(resource_path='/com/github/rotlug/Freebie/gtk/browse_view.ui')
class BrowseView(Gtk.Box):
    __gtype_name__ = "BrowseView"

    library: Gtk.FlowBox = Gtk.Template.Child()

    def __init__(self, search_entry: Gtk.SearchEntry, stack: Adw.ViewStack, nav_view: Adw.NavigationView, **kwargs):
        super().__init__(**kwargs)
        self.search_entry = search_entry
        self.search_entry.connect("search_changed", self.on_search_entry_search_changed)
        self.stack = stack
        self.nav_view = nav_view
        
    def on_search_entry_search_changed(self, widget: Gtk.SearchEntry):
        text = widget.get_text()
        if self.stack.get_visible_child_name() != "browse": return
        if len(text) < 3 and text != "":
            return
        
        thread = Thread(target=self.populate_library, daemon=True, args=[text])
        thread.start()
        self.library.remove_all()

    def populate_library(self, text):
        games = game_manager.search(text)
        for game in games:
            if not self.eligible_to_search(text):
                print("Search Aborted!")
                return
            
            game.metadata = igdb.search(game)
            if (game.metadata != None) and self.eligible_to_search(text): GLib.idle_add(self.add_game_to_library, game)
            
            """
            The reason that there is check for [eligible_to_search] twice in
            This function is because if igdb.search() takes a while to execute (for example, if the game isn't in the cache)
            then that value might change by the time that it completes.
            """

    def eligible_to_search(self, search_term):
        return (self.search_entry.get_text() == search_term)

    def add_game_to_library(self, game):
        box = GameBox(game)
        
        box.connect(self.select_game) # type: ignore
        self.library.append(box) # type: ignore
    
    def select_game(self, widget, game: Game):
        self.nav_view.push_by_tag("game")
        page: GamePage = self.nav_view.find_page("game") # type: ignore
        page.set_game(game)