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

import os
from gi.repository import Adw
from gi.repository import Gtk

from .backend.ensure import DATA_DIR, is_non_empty_directory

from .pages.main_page import MainPage
from .pages.game_page import GamePage
from .pages.proton_page import ProtonPage
from .pages.igdb_page import IGDBPage

@Gtk.Template(resource_path='/com/github/rotlug/Freebie/gtk/window.ui')
class FreebieWindow(Adw.ApplicationWindow):
    __gtype_name__ = 'FreebieWindow'

    nav_view: Adw.NavigationView = Gtk.Template.Child()
    
    def __init__(self, application: Adw.Application):
        super().__init__(application=application)
        
        if not is_non_empty_directory(f"{DATA_DIR}/proton") or not is_non_empty_directory(f"{DATA_DIR}/prefix"):
            self.nav_view.add(ProtonPage(self.nav_view)) #type: ignore
        elif not os.path.exists(f"{DATA_DIR}/igdb.txt"):
            self.nav_view.add(IGDBPage(self.nav_view))
        else:
            self.nav_view.add(MainPage(self.nav_view, self))
            self.nav_view.add(GamePage(self.nav_view, self))
