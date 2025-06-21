
from typing import Any
from gi.repository import Adw, Gtk
from requests import patch

from freebie.backend.igdb_api import igdb
from freebie.backend.game import Game
from freebie.game_manager import game_manager


@Gtk.Template(resource_path='/com/github/rotlug/Freebie/gtk/add_game_dialog.ui')
class AddGameDialog(Adw.Dialog):

    __gtype_name__ = "AddGameDialog"

    add_button: Gtk.Button = Gtk.Template.Child()
    cancel_button: Gtk.Button = Gtk.Template.Child()
    exe_file_chooser_button: Gtk.Button = Gtk.Template.Child()

    select_exe_row: Adw.ActionRow = Gtk.Template.Child()
    game_name_row: Adw.ActionRow = Gtk.Template.Child()

    def __init__(self, window: Gtk.Window, **kwargs: Any):
        super().__init__(**kwargs)

        self.add_button.connect("clicked", self.on_add_button_clicked)
        self.cancel_button.connect("clicked", self.on_cancel_button_clicked)
        self.exe_file_chooser_button.connect("clicked", self.choose_exe_location)
        self.game_name_row.connect("changed", self.on_game_name_changed)

        self.window = window
        self.path: str = ""
        self.game_name = ""

    def on_game_name_changed(self, widget: Adw.EntryRow):
        self.add_button.set_sensitive(widget.get_text_length() > 0)
        self.game_name = widget.get_text()

    def choose_exe_location(self, _):
        dialog = Gtk.FileChooserNative(
            title="Choose Executable",
            action=Gtk.FileChooserAction.OPEN,
            transient_for=self.window,
            modal=True,
            filter=Gtk.FileFilter(mime_types=["application/x-msdownload"])
        )

        dialog.connect("response", self.on_file_selected)
        dialog.show()

    def on_file_selected(self, widget: Gtk.FileChooserNative, _):
        path: str = widget.get_file().get_path() # type: ignore
        self.path = path
        self.select_exe_row.set_subtitle(path)

    def on_add_button_clicked(self, _):
        game = Game(self.game_name, "", "")
        game.metadata = igdb.search(game)

        # Add game to installed games
        game_manager.add_custom_game_to_installed(game, self.path)

        self.close()

    def on_cancel_button_clicked(self, _):
        self.close()

