from typing import Any
from gi.repository import Adw, Gtk

from freebie.backend.igdb_api import igdb
from freebie.backend.game import InstalledGame
from freebie.util.game_manager import game_manager


@Gtk.Template(resource_path="/com/github/rotlug/Freebie/gtk/add_game_dialog.ui")
class AddGameDialog(Adw.Dialog):
    __gtype_name__ = "AddGameDialog"

    add_button: Gtk.Button = Gtk.Template.Child()
    cancel_button: Gtk.Button = Gtk.Template.Child()
    exe_file_chooser_button: Gtk.Button = Gtk.Template.Child()

    select_exe_row: Adw.ActionRow = Gtk.Template.Child()
    game_name_row: Adw.EntryRow = Gtk.Template.Child()

    def __init__(self, window: Gtk.Window, **kwargs: Any):
        super().__init__(**kwargs)

        self.add_button.connect("clicked", self.on_add_button_clicked)
        self.cancel_button.connect("clicked", self.on_cancel_button_clicked)
        self.exe_file_chooser_button.connect("clicked", self.choose_exe_location)

        self.game_name_row.connect("changed", self.on_game_name_changed)

        self.window = window
        self.path: str = ""
        self.game_name = ""

    def on_data_changed(self):
        print(self.game_name)
        print(self.path)

        can_add_game = len(self.game_name) > 0 and len(self.path) > 0

        self.add_button.set_sensitive(can_add_game)

    def on_game_name_changed(self, widget: Adw.EntryRow):
        self.game_name = widget.get_text()
        self.on_data_changed()

    def choose_exe_location(self, _):
        dialog = Gtk.FileChooserNative(
            title=_("Choose Executable"),
            action=Gtk.FileChooserAction.OPEN,
            transient_for=self.window,
            modal=True,
            filter=Gtk.FileFilter(mime_types=["application/x-msdownload"]),
        )

        dialog.connect("response", self.on_file_selected)
        dialog.show()

    def on_file_selected(self, widget: Gtk.FileChooserNative, _):
        f = widget.get_file()
        if f is None:
            return

        path = f.get_path()
        if path is None:
            return

        self.path = path
        self.select_exe_row.set_subtitle(path)

        self.on_data_changed()

    def on_add_button_clicked(self, _):
        game = InstalledGame(self.game_name, exe=self.path, directory="")
        game.metadata = igdb.search(game)
        if game.metadata is not None:
            game.name = game.metadata.name

        # Add game to installed games
        game_manager.add_custom_game_to_installed(game)

        self.close()

    def on_cancel_button_clicked(self, _):
        self.close()
