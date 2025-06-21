from subprocess import call
from threading import Thread
import time
from typing import Any

from .backend.utils import umu_run
from .backend.fitgirl.installer import FitgirlInstaller
from .backend.fitgirl.provider import FitgirlProvider
from .backend.game import Game, InstalledGame
from .backend.provider import Installer, Provider

from .backend.ensure import ensure_directory, DATA_DIR, find

from .backend import json_utils

from gi.repository import Gtk, Adw, GLib

from datetime import timedelta

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
        self.play_view: Any = None # typeof PlayView
        self.game_page: Any = None # typeof GamePage
    
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

    def add_custom_game_to_installed(self, game: InstalledGame):
        json_utils.add_to_file(
            f"{DATA_DIR}/installed.json",
            game.name,
            game.to_dict()
        )

        if self.play_view != None:
            self.play_view.update_game_array()

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
            time.sleep(3)

        else:
            del self.game_statuses[game_slug]
        
        if self.play_view != None:
            self.play_view.update_game_array()
        if self.game_page != None:
            self.game_page.set_game(game)
    
    def update_button_task(self, game_page, nav: Adw.NavigationView):
        while True:
            page = nav.get_visible_page()
            if (page != None) and page.get_tag() == "game":
                GLib.idle_add(self.update_button, game_page.game, game_page.action_button)
            time.sleep(3)
        
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
        
        installed_game = InstalledGame.from_dict(game.name, installed[game.name])

        start_time = time.time() # Track number of seconds that game has been open

        umu_run(f'"{installed_game.exe}"')

        seconds_elapsed = int(time.time() - start_time)

        installed_game.seconds_played += seconds_elapsed

        json_utils.add_to_file(f"{DATA_DIR}/installed.json", installed_game.name, installed_game.to_dict())

        if (self.game_page is not None):
            installed_game.metadata
            self.game_page.set_game(installed_game) # Update with current time played

        del self.game_statuses[game.get_slug(True)] # Remove "Running" status from the game

    def get_button_target(self, game: Game):
        if self.is_installed(game): return self.run_game_thread
        return self.get_game_thread
    
    def uninstall(self, game: Game):
        call(f"rm -r {json_utils.get_file(f"{DATA_DIR}/installed.json")[game.name]["dir"]}", shell=True)
        json_utils.remove_from_file(f"{DATA_DIR}/installed.json", game.name)
        if self.play_view != None:
            self.play_view.update_game_array()
        
    def is_installed(self, game: Game):
        return game.name in json_utils.get_file(f"{DATA_DIR}/installed.json")

    def format_duration(self, seconds: int) -> str:
        delta = timedelta(seconds=seconds)
        days = delta.days
        hours, remainder = divmod(delta.seconds, 3600)
        minutes, seconds = divmod(remainder, 60)

        parts = []
        if days:
            parts.append(f"{days} day{'s' if days != 1 else ''}")
        if hours:
            parts.append(f"{hours} hour{'s' if hours != 1 else ''}")
        if minutes:
            parts.append(f"{minutes} minute{'s' if minutes != 1 else ''}")
        if seconds:
            parts.append(f"{seconds} second{'s' if seconds != 1 else ''}")

        return ", ".join(parts[:-1]) + (" and " + parts[-1] if len(parts) > 1 else parts[0]) if parts else "0 seconds"

    def get_all_installed_games(self):
        installed = json_utils.get_file(f"{DATA_DIR}/installed.json")

        games: list[InstalledGame] = []
        for g in installed.keys():
            name: str = g

            obj = installed[g]

            exe: str = obj.get("exe", "")
            directory: str = obj.get("dir", "")
            seconds_played: int = obj.get("seconds_played", 0)

            game = InstalledGame(name, exe, directory)
            game.seconds_played = seconds_played

            games.append(game)

        return games

# GameManager singleton
ensure_directory("downloads")
game_manager = GameManager()
