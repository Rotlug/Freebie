
from typing import Any
from gi.repository import Adw, Gtk


@Gtk.Template(resource_path='/com/github/rotlug/Freebie/gtk/add_game_dialog.ui')
class AddGameDialog(Adw.Dialog):

    __gtype_name__ = "AddGameDialog"

    def __init__(self, **kwargs: Any):
        super().__init__(**kwargs)
