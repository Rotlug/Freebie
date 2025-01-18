# main.py
#
# Copyright 2024 rotlug
#
# This program is free software: you can redistribute it and/or modify
# it under the terms of the GNU General Public License as published by
# the Free Software Foundation, either version 3 of the License, or
# (at your option) any later version.
#
# This program is distributed in the hope that it will be useful,
# but WITHOUT ANY WARRANTY; without even the implied warranty of
# MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
# GNU General Public License for more details.
#
# You should have received a copy of the GNU General Public License
# along with this program.  If not, see <https://www.gnu.org/licenses/>.
#
# SPDX-License-Identifier: GPL-3.0-or-later

from threading import Thread
import sys
import gi
from .backend.utils import umu_run
from .backend.igdb_api import igdb
from .backend.fitgirl.installer import proc

gi.require_version('Gtk', '4.0')
gi.require_version('Adw', '1')

from gi.repository import Gtk, Gio, Adw
from .window import FreebieWindow

class FreebieApplication(Adw.Application):
    """The main application singleton class."""

    def __init__(self):
        super().__init__(application_id='com.github.rotlug.Freebie',
                         flags=Gio.ApplicationFlags.DEFAULT_FLAGS)
        self.create_action('quit', lambda *_: self.quit, ['<primary>q'])
        self.create_action('about', self.on_about_action)
        self.create_action('preferences', self.on_preferences_action)
        self.create_action('run_exe', self.on_run_exe_action, ['<primary>e'])
        
        Thread(target=igdb.save_cache_task, name="SaveMetadata", daemon=True).start()
        print("HEllo world")

    def do_activate(self):
        """Called when the application is activated.

        We raise the application's main window, creating it if
        necessary.
        """
        win = self.get_active_window()
        if win == None:
            win = FreebieWindow(self) # type: ignore
        
        win.present() # type: ignore
    
    def on_run_exe_action(self, widget, _):
        dialog = Gtk.FileChooserNative(
            title="Choose Executable",
            action=Gtk.FileChooserAction.OPEN,
            transient_for=self.get_active_window(),
            modal=True,
            filter=Gtk.FileFilter(mime_types=["application/x-msdownload"])
        )
        
        dialog.connect("response", self.on_exe_file_selected)
        dialog.show()

    def on_exe_file_selected(self, widget: Gtk.FileChooserNative, _):
        path: str = widget.get_file().get_path() # type: ignore
        print(f"Running exe: {path}")
        Thread(target=umu_run, daemon=True, args=[f"'{path}'"]).start()
        widget.destroy()
    
    def on_about_action(self, widget, _):
        """Callback for the app.about action."""
        about = Adw.AboutDialog(
                                application_name='Freebie',
                                application_icon='com.github.rotlug.Freebie',
                                developer_name='rotlug',
                                version='0.1.0',
                                developers=['rotlug'],
                                copyright='© 2024 rotlug')
        about.present(self.get_active_window())

    def on_preferences_action(self, widget, _):
        """Callback for the app.preferences action."""
        print('app.preferences action activated')

    def create_action(self, name, callback, shortcuts: list[str] | None = None):
        """Add an application action.

        Args:
            name: the name of the action
            callback: the function to be called when the action is
              activated
            shortcuts: an optional list of accelerators
        """
        action = Gio.SimpleAction.new(name, None)
        action.connect("activate", callback)
        self.add_action(action)

        if shortcuts:
            self.set_accels_for_action(f"app.{name}", shortcuts)

def main(version):
    # ensure.ensure_wine_prefix() # Make sure that a wine prefix exists
    """The application's entry point."""
    app = FreebieApplication()
    return_code = app.run(sys.argv)
    
    # Save cache to disk and quit
    igdb.save_cache_to_disk()

    # Kill aria2p
    proc.kill()

    return return_code