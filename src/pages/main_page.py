from gi.repository import Adw
from gi.repository import Gtk
from ..game_manager import GameManager
from .browse_view import BrowseView

from ..backend.igdb_api import igdb

@Gtk.Template(resource_path='/com/github/rotlug/Freebie/gtk/main_page.ui')
class MainPage(Adw.NavigationPage):
    __gtype_name__ = "MainPage"
    searchbar: Gtk.SearchBar = Gtk.Template.Child()
    search_entry: Gtk.SearchEntry = Gtk.Template.Child()
    search_button: Gtk.ToggleButton = Gtk.Template.Child()
    stack: Adw.ViewStack = Gtk.Template.Child()

    browse: Gtk.Box = Gtk.Template.Child()

    def __init__(self, nav: Adw.NavigationView, **kwargs):
        super().__init__(**kwargs)
                
        self.nav = nav
        self.searchbar.connect("notify::search-mode-enabled", self.on_searchbar_toggled)
        self.search_button.connect("toggled", self.on_toggle_search_action)

        # self.browse.append(BrowsePage(self.searchentry, self.stack)) # Browse Page
        self.stack.connect("notify::visible-child", self.retract_search_bar)
        self.browse.append(BrowseView(self.search_entry, self.stack)) # type: ignore
    
    def retract_search_bar(self, widget, _):
        self.search_button.set_active(False)

    def on_searchbar_toggled(self, widget, paramspec):
        enabled = self.searchbar.get_search_mode()
        self.search_button.set_active(enabled)

    def on_toggle_search_action(self, widget, _=None):
        toggled = self.search_button.get_active()
        self.searchbar.set_search_mode(toggled)