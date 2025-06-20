from typing import Any
from gi.repository import Adw, Gtk

from ..backend.igdb_api import igdb

@Gtk.Template(resource_path='/com/github/rotlug/Freebie/gtk/preferences.ui')
class FreebiePreferences(Adw.PreferencesDialog):
    __gtype_name__ = "FreebiePreferences"

    igdb_client_id_entry: Adw.EntryRow = Gtk.Template.Child()
    igdb_secret_entry: Adw.PasswordEntryRow = Gtk.Template.Child()

    def __init__(self, **kwrags: Any):
        super().__init__(**kwrags)

        igdb.get_credentials()

        self.igdb_client_id_entry.set_text(igdb.client_id)
        self.igdb_secret_entry.set_text(igdb.secret)

        self.igdb_client_id_entry.connect("apply", self.on_client_id_changed)
        self.igdb_secret_entry.connect("apply", self.on_secret_changed)

    def on_client_id_changed(self, widget: Adw.EntryRow):
        # Update igdb client id
        igdb.client_id = ""
        igdb.update_igdb_credentials_file(widget.get_text(), igdb.secret)

    def on_secret_changed(self, widget: Adw.PasswordEntryRow):
        # Update igdb secret
        igdb.secret = ""
        igdb.update_igdb_credentials_file(igdb.client_id, widget.get_text())

