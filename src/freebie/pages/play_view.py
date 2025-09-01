from gi.repository import Adw, Gtk, GLib

from freebie.widgets.game_box import GameBox
from .game_page import GamePage
from ..backend.game import Game, InstalledGame

from freebie.util.game_manager import game_manager
from ..backend.igdb_api import igdb

from ..backend.ensure import ensure_file


@Gtk.Template(resource_path="/com/github/rotlug/Freebie/gtk/play_view.ui")
class PlayView(Gtk.Box):
    __gtype_name__ = "PlayView"

    library: Gtk.FlowBox = Gtk.Template.Child()
    library_stack: Gtk.Stack = Gtk.Template.Child()

    def __init__(
        self,
        search_entry: Gtk.SearchEntry,
        stack: Adw.ViewStack,
        nav_view: Adw.NavigationView,
        **kwargs,
    ):
        super().__init__(**kwargs)
        self.search_entry = search_entry
        self.search_entry.connect("changed", self.on_search_entry_search_changed)
        self.stack = stack
        self.nav_view = nav_view

        game_manager.play_view = self
        self.library_stack.set_transition_type(Gtk.StackTransitionType.CROSSFADE)

        ensure_file("installed.json", "{}")
        self.games: list[InstalledGame] = []
        self.update_game_array()

    def update_game_array(self):
        self.games = []

        for game in game_manager.get_all_installed_games():
            game.metadata = igdb.search(game)

            if game.metadata is None:
                print(
                    f"Warning: {game.name} doesn't have metadata. skipped showing in play view'"
                )
                continue

            game.name = game.metadata.name

            self.games.append(game)

        GLib.idle_add(self.library.remove_all)

        if len(self.games) == 0:
            self.library_stack.set_visible_child(self.library_stack.get_last_child())  # type: ignore
        else:
            self.library_stack.set_visible_child(self.library_stack.get_first_child())  # type: ignore

        for game in self.games:
            self.add_game_to_library(game)

    def on_search_entry_search_changed(self, widget: Gtk.SearchEntry):
        self.text = widget.get_text()
        new_games = filter(self.filter_game, self.games)
        GLib.idle_add(self.library.remove_all)

        for game in new_games:
            self.add_game_to_library(game)

    def filter_game(self, game) -> bool:
        return self.search_entry.get_text().lower() in game.name.lower()

    def add_game_to_library(self, game):
        box = GameBox(game)

        box.connect_button(self.select_game)  # type: ignore
        GLib.idle_add(self.library.append, box)  # type: ignore

    def select_game(self, widget, game: Game):
        self.nav_view.push_by_tag("game")
        page: GamePage = self.nav_view.find_page("game")  # type: ignore
        page.set_game(game)
