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
from .backend.igdb_api import igdb
from .game_manager import game_manager

from .pages.main_page import MainPage
from .pages.game_page import GamePage

@Gtk.Template(resource_path='/com/github/rotlug/Freebie/gtk/window.ui')
class FreebieWindow(Adw.ApplicationWindow):
    __gtype_name__ = 'FreebieWindow'

    nav_view: Adw.NavigationView = Gtk.Template.Child()
    
    def __init__(self, application: Adw.Application):
        super().__init__(application=application)
        self.nav_view.add(MainPage(self.nav_view)) #type: ignore
        self.nav_view.add(GamePage()) # type: ignore
    