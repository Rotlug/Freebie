from gi.repository import Adw, Gtk
from ..backend.utils import restart
from ..backend.ensure import DATA_DIR


@Gtk.Template(resource_path="/com/github/rotlug/Freebie/gtk/igdb_page.ui")
class IGDBPage(Adw.NavigationPage):
    __gtype_name__ = "IGDBPage"

    secret_entry: Adw.EntryRow = Gtk.Template.Child()
    client_id_entry: Adw.PasswordEntryRow = Gtk.Template.Child()

    subtitle: Gtk.Label = Gtk.Template.Child()
    apply: Gtk.Button = Gtk.Template.Child()

    def __init__(self, nav: Adw.NavigationView, **kwrags):
        self.nav = nav
        super().__init__(**kwrags)

        self.subtitle.set_markup(
            'You can get your credentials using <a href="https://api-docs.igdb.com/#account-creation">this guide</a>.'
        )

        self.client_id_entry.connect("changed", self.update_apply_button)
        self.secret_entry.connect("changed", self.update_apply_button)

        self.update_apply_button("")

        self.apply.connect("clicked", self.on_apply_click)

    def on_apply_click(self, _):
        with open(f"{DATA_DIR}/igdb.txt", "w") as f:
            f.writelines(
                [self.client_id_entry.get_text() + "\n", self.secret_entry.get_text()]
            )

        restart()

    def is_valid(self):
        return (
            self.secret_entry.get_text_length() == 30
            and self.client_id_entry.get_text_length() == 30
        )

    def update_apply_button(self, _):
        self.apply.set_sensitive(self.is_valid())
