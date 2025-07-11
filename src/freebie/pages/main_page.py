from gi.repository import Adw
from gi.repository import Gtk
from .browse_view import BrowseView
from .play_view import PlayView


@Gtk.Template(resource_path='/com/github/rotlug/Freebie/gtk/main_page.ui')
class MainPage(Adw.NavigationPage):
    __gtype_name__ = "MainPage"
    searchbar: Gtk.SearchBar = Gtk.Template.Child()
    search_entry: Gtk.SearchEntry = Gtk.Template.Child()
    search_button: Gtk.ToggleButton = Gtk.Template.Child()

    stack: Adw.ViewStack = Gtk.Template.Child()
    
    browse: Gtk.Box = Gtk.Template.Child()
    play: Gtk.Box = Gtk.Template.Child()

    def __init__(self, nav: Adw.NavigationView, win: Gtk.Window, **kwargs):
        super().__init__(**kwargs)

        self.browse_view_opened = False # Keep track if the browse view has been opened in this session

        self.nav = nav
        self.win = win

        self.searchbar.set_key_capture_widget(self.win)
        self.searchbar.connect_entry(self.search_entry)

        self.searchbar.connect("notify::search-mode-enabled", self.on_searchbar_toggled)
        self.search_button.connect("toggled", self.on_toggle_search_action)

        self.stack.connect("notify::visible-child", self.visible_child_changed)

        # Add Views
        self.browse_view = BrowseView(self.search_entry, self.stack, self.nav)
        self.browse.append(self.browse_view)

        self.play.append(PlayView(self.search_entry, self.stack, self.nav))

        # Trigger visible_child_changed at startup
        self.visible_child_changed(None, None)

    def visible_child_changed(self, _, __):
        self.search_button.set_active(False)
        
        name = self.stack.get_visible_child().get_first_child().get_name() # type: ignore
        
        if name == "BrowseView":
            if not self.browse_view_opened:
                self.search_entry.connect("changed", self.browse_view.on_search_entry_search_changed)
                self.browse_view.populate_library(self.search_entry.get_text())

            self.browse_view_opened = True

    def on_searchbar_toggled(self, widget, paramspec):
        enabled = self.searchbar.get_search_mode()
        self.search_button.set_active(enabled)

    def on_toggle_search_action(self, widget, _=None):
        toggled = self.search_button.get_active()
        self.searchbar.set_search_mode(toggled)
