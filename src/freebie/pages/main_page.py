from gi.repository import Adw
from gi.repository import Gtk
# from ..game_manager import GameManager
from .browse_view import BrowseView
from .play_view import PlayView

# from ..backend.igdb_api import igdb

@Gtk.Template(resource_path='/com/github/rotlug/Freebie/gtk/main_page.ui')
class MainPage(Adw.NavigationPage):
    __gtype_name__ = "MainPage"
    searchbar: Gtk.SearchBar = Gtk.Template.Child()
    search_entry: Gtk.SearchEntry = Gtk.Template.Child()
    search_button: Gtk.ToggleButton = Gtk.Template.Child()

    stack: Adw.ViewStack = Gtk.Template.Child()
    
    browse: Gtk.Box = Gtk.Template.Child()
    play: Gtk.Box = Gtk.Template.Child()

    def __init__(self, nav: Adw.NavigationView, **kwargs):
        super().__init__(**kwargs)
                
        self.nav = nav
        self.searchbar.connect("notify::search-mode-enabled", self.on_searchbar_toggled)
        self.search_button.connect("toggled", self.on_toggle_search_action)

        self.stack.connect("notify::visible-child", self.visible_child_changed)

        # Add Views
        self.browse.append(BrowseView(self.search_entry, self.stack, self.nav)) # type: ignore
        self.play.append(PlayView(self.search_entry, self.stack, self.nav)) # type: ignore
    
    def visible_child_changed(self, widget, _):
        self.search_button.set_active(False)
        
        name = self.stack.get_visible_child().get_first_child().get_name() # type: ignore
        
        if name == "BrowseView":
            self.search_entry.set_search_delay(500)
        else:
            self.search_entry.set_search_delay(0)
        
    def on_searchbar_toggled(self, widget, paramspec):
        enabled = self.searchbar.get_search_mode()
        self.search_button.set_active(enabled)

    def on_toggle_search_action(self, widget, _=None):
        toggled = self.search_button.get_active()
        self.searchbar.set_search_mode(toggled)
