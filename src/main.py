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
from .backend.igdb_api import igdb

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
        
        Thread(target=igdb.save_cache_task, name="SaveMetadata", daemon=True).start()
    
    def do_activate(self):
        """Called when the application is activated.

        We raise the application's main window, creating it if
        necessary.
        """
        win = self.get_active_window()
        if win == None:
            win = FreebieWindow(self) # type: ignore
        
        win.present() # type: ignore
    
    def on_about_action(self, widget, _):
        """Callback for the app.about action."""
        about = Adw.AboutDialog(
                                application_name='freebie',
                                application_icon='com.github.rotlug.Freebie',
                                developer_name='rotlug',
                                version='0.1.0',
                                developers=['rotlug'],
                                copyright='© 2024 rotlug')
        about.present(self.get_active_window())

    def on_preferences_action(self, widget, _):
        """Callback for the app.preferences action."""
        print('app.preferences action activated')

    def create_action(self, name, callback, shortcuts=None):
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
    """The application's entry point."""
    app = FreebieApplication()
    return_code = app.run(sys.argv)
    
    # Save cache to disk and quit
    igdb.save_cache_to_disk()
    return return_code