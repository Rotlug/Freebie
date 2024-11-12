from threading import Thread
from gi.repository import Adw, Gtk, GLib
from .game_page import GamePage
from ..backend.game import Game

from ..game_manager import game_manager
from ..backend.igdb_api import igdb

from ..game_box import GameBox
from ..backend import json_utils
from ..backend.ensure import DATA_DIR, ensure_file

@Gtk.Template(resource_path='/com/github/rotlug/Freebie/gtk/play_view.ui')
class PlayView(Gtk.Box):
    __gtype_name__ = "PlayView"

    library: Gtk.FlowBox = Gtk.Template.Child()

    def __init__(self, search_entry: Gtk.SearchEntry, stack: Adw.ViewStack, nav_view: Adw.NavigationView, **kwargs):
        super().__init__(**kwargs)
        self.search_entry = search_entry
        self.search_entry.connect("search_changed", self.on_search_entry_search_changed)
        self.stack = stack
        self.nav_view = nav_view
        
        game_manager.play_view = self
        
        ensure_file("installed.json", "{}")
        self.games = []
        self.update_game_array()
    
    def update_game_array(self):
        self.games = []

        for g in json_utils.get_file(f"{DATA_DIR}/installed.json").keys():
            print(g)
            game = Game(g, "", "")
            game.metadata = igdb.search(game)
            self.games.append(game)

        GLib.idle_add(self.library.remove_all)

        for game in self.games:
            self.add_game_to_library(game)
        
    def on_search_entry_search_changed(self, widget: Gtk.SearchEntry):
        self.text = widget.get_text()
        new_games = filter(self.filter_game, self.games)
        GLib.idle_add(self.library.remove_all)

        for game in new_games:
            self.add_game_to_library(game)
    
    def filter_game(self, game) -> bool:
        if self.search_entry.get_text().lower() in game.name.lower():
            return True
        else:
            return False
    
    def eligible_to_search(self, search_term):
        return (self.search_entry.get_text() == search_term)

    def add_game_to_library(self, game):
        box = GameBox(game)

        box.connect(self.select_game) # type: ignore
        GLib.idle_add(self.library.append, box) # type: ignore
    
    def select_game(self, widget, game: Game):
        self.nav_view.push_by_tag("game")
        page: GamePage = self.nav_view.find_page("game") # type: ignore
        page.set_game(game)

