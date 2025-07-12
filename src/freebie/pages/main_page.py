import os
from gi.repository import Adw
from gi.repository import Gtk

from freebie.backend.utils import DATA_DIR
from .browse_view import BrowseView
from .play_view import PlayView

import atexit


@Gtk.Template(resource_path="/com/github/rotlug/Freebie/gtk/main_page.ui")
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

        self.browse_view_opened = (
            False  # Keep track if the browse view has been opened in this session
        )

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

        # Set visible view to the view from last session
        view_saver = CurrentViewSaver(self.stack)
        self.stack.set_visible_child_name(view_saver.visible_child)

        # Trigger visible_child_changed at startup
        self.visible_child_changed(None, None, False)

    def visible_child_changed(self, _, __, is_user_action=True):
        self.search_button.set_active(False)

        name = self.stack.get_visible_child_name()
        if name is None:
            return

        if name == "browse":
            if not self.browse_view_opened:
                self.search_entry.connect(
                    "search_changed", self.browse_view.on_search_entry_search_changed
                )

                if is_user_action:
                    self.search_entry.emit("search_changed")

            self.browse_view_opened = True

    def on_searchbar_toggled(self, widget, paramspec):
        enabled = self.searchbar.get_search_mode()
        self.search_button.set_active(enabled)

    def on_toggle_search_action(self, widget, _=None):
        toggled = self.search_button.get_active()
        self.searchbar.set_search_mode(toggled)


class CurrentViewSaver:
    SAVE_FILE = f"{DATA_DIR}/last_view.txt"

    def __init__(self, stack: Adw.ViewStack) -> None:
        self.stack = stack
        self.stack.connect("notify::visible-child", self.on_visible_child_changed)

        self.visible_child = "browse"  # default to browse

        atexit.register(self.exit_handler)

        if os.path.exists(CurrentViewSaver.SAVE_FILE):
            with open(CurrentViewSaver.SAVE_FILE, "r") as f:
                self.visible_child = f.read()

    def on_visible_child_changed(self, stack: Adw.ViewStack, _):
        name = stack.get_visible_child_name()
        if name is None:
            return

        self.visible_child = name

    def exit_handler(self):
        with open(CurrentViewSaver.SAVE_FILE, "w") as f:
            f.write(self.visible_child)
