from threading import Thread
from time import sleep
from .backend.fitgirl.installer import FitgirlInstaller
from .backend.fitgirl.provider import FitgirlProvider
from .backend.game import Game
from .backend.provider import Installer, Provider

from gi.repository import Gtk

class Source:
    def __init__(self, provider: type[Provider], installer: type[Installer]) -> None:
        self.provider = provider()
        self.installer = installer()

class GameManager:
    def __init__(self) -> None:
        self.sources = [
            Source(FitgirlProvider, FitgirlInstaller)
        ]

        self.game_statuses: dict[str, str] = {}
    
    def search(self, query: str) -> list[Game]:
        games: list[Game] = []
        for source in self.sources:
            games += source.provider.search(query)
        
        return self.remove_games_without_pictures(games)

    def remove_games_without_pictures(self, games: list[Game]):
        new_games = []
        for game in games:
            if game.metadata != None: new_games.append(game)
        
        return new_games
    
    def get_popular(self) -> list[Game]:
        games = []
        for source in self.sources:
            games += source.provider.get_popular()
        return self.remove_games_without_pictures(games)

    def get_game(self, game: Game):
        installer = game.installer()
        installer_thread = Thread(target=installer.get_game, args=[game], daemon=True)
        installer_thread.start()

        game_slug = game.get_slug(True)
        
        self.game_statuses[game_slug] = "Downloading... 0%"
        while installer_thread.is_alive():
            if game_slug in installer.downloads:
                self.game_statuses[game_slug] = f"Downloading... {installer.downloads[game_slug].progress_string()}"
            sleep(3)
    
    def update_button_task(self, game: Game, button: Gtk.Button):
        while True:
            self.update_button(game, button)
            sleep(3)
        
    def update_button(self, game: Game, button: Gtk.Button):
        slug = game.get_slug(True)
        if slug in self.game_statuses:
            button.set_sensitive(False)
            button.set_css_classes([])
            button.remove_css_class("suggested-action")
            button.set_label(self.game_statuses[slug])
        else:
            button.set_sensitive(True)
            button.set_label(f"Get ({game.size})")
            button.add_css_class("suggested-action")

# GameManager singleton
game_manager = GameManager()