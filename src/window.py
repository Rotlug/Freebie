# window.py
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

from gi.repository import Adw
from gi.repository import Gtk
from .game_manager import GameManager
from .pages.main_view import MainView

@Gtk.Template(resource_path='/com/github/rotlug/Freebie/gtk/window.ui')
class FreebieWindow(Adw.ApplicationWindow):
    __gtype_name__ = 'FreebieWindow'

    nav_view: Adw.NavigationView = Gtk.Template.Child()
    game_manager = GameManager()
    
    def __init__(self, application: Adw.Application):
        super().__init__(application=application)
        self.nav_view.add(MainView(self.nav_view)) #type: ignore