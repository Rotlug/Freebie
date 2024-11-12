from subprocess import call
from threading import Thread
from time import sleep
from typing import Any
from .backend.fitgirl.installer import FitgirlInstaller
from .backend.fitgirl.provider import FitgirlProvider
from .backend.game import Game
from .backend.provider import Installer, Provider

from .backend.ensure import ensure_directory, DATA_DIR

from .backend import json_utils

from gi.repository import Gtk, Adw, GLib

class Source:
    def __init__(self, provider: type[Provider], installer: type[Installer]):
        self.provider = provider()
        self.installer = installer()

class GameManager:
    def __init__(self) -> None:
        self.sources = [
            Source(FitgirlProvider, FitgirlInstaller)
        ]

        self.game_statuses: dict[str, str] = {}
        self.play_view: Any
    
    def search(self, query: str) -> list[Game]:
        if query == "": return self.get_popular()
        
        games: list[Game] = []
        for source in self.sources:
            games += source.provider.search(query)

        return games

    def remove_games_without_pictures(self, games: list[Game]):
        new_games = []
        for game in games:
            if game.metadata != None: new_games.append(game)
        
        return new_games
    
    def get_popular(self) -> list[Game]:
        games = []
        for source in self.sources:
            games += source.provider.get_popular()

        return games

    def get_game(self, game: Game):
        installer = game.installer()
        installer_thread = Thread(target=installer.get_game, args=[game], daemon=True)
        installer_thread.start()

        game_slug = game.get_slug(True)
        
        self.game_statuses[game_slug] = "Downloading... 0%"
        while installer_thread.is_alive():
            if game_slug in installer.downloads:
                self.game_statuses[game_slug] = f"Downloading... {installer.downloads[game_slug].progress_string()}"
            else:
                self.game_statuses[game_slug] = "Installing..."
            sleep(3)
        else:
            del self.game_statuses[game_slug]
        
        if self.play_view != None:
            self.play_view.update_game_array()

    def update_button_task(self, game_page, nav: Adw.NavigationView):
        while True:
            page = nav.get_visible_page()
            if (page != None) and page.get_tag() == "game":
                GLib.idle_add(self.update_button, game_page.game, game_page.action_button)
            sleep(3)
        
    def update_button(self, game: Game, button: Gtk.Button):
        slug = game.get_slug(True)
        if slug in self.game_statuses:
            button.set_sensitive(False)
            button.set_css_classes([])
            button.remove_css_class("suggested-action")
            button.set_label(self.game_statuses[slug])
        else:
            if game.name in json_utils.get_file(f"{DATA_DIR}/installed.json"):
                button.set_sensitive(True)
                button.set_label("Play")
                button.add_css_class("suggested-action")
            else:
                button.set_sensitive(True)
                button.set_label(f"Get ({game.size})")
                button.add_css_class("suggested-action")    
    
    def get_game_thread(self, game: Game): Thread(target=self.get_game, args=[game], daemon=True).start()
    def run_game_thread(self, game: Game): Thread(target=self.run_game, args=[game], daemon=True).start()

    def run_game(self, game: Game):
        self.game_statuses[game.get_slug(True)] = "Running"
        installed = json_utils.get_file(f"{DATA_DIR}/installed.json")
        if game.name not in installed: return
        
        command = installed[game.name]
        print(f"COMMAND: {command}")
        call(command, shell=True)
        del self.game_statuses[game.get_slug(True)]
    
    def get_button_target(self, game: Game):
        installed = json_utils.get_file(f"{DATA_DIR}/installed.json")
        if game.name in installed: return self.run_game_thread
        return self.get_game_thread
    
# GameManager singleton
ensure_directory("downloads")
game_manager = GameManager()