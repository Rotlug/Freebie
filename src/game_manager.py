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

# GameManager singleton
class GameManager:
    def __init__(self) -> None:
        self.sources = [
            Source(FitgirlProvider, FitgirlInstaller)
        ]

        self.game_statuses: dict[str, str] = {}
    
    def search(self, query: str) -> list[Game]:
        games = []
        for source in self.sources:
            games += source.provider.search(query)
        return games

    def get_popular(self) -> list[Game]:
        games = []
        for source in self.sources:
            games += source.provider.get_popular()
        return games

    def get_game(self, game: Game):
        installer = game.installer()
        installer_thread = Thread(target=installer.get_game, args=[game])
        installer_thread.start()

        game_slug = game.get_slug(True)
        while installer_thread.is_alive():
            if game_slug in installer.downloads:
                self.game_statuses[game_slug] = f"Downloading... {installer.downloads[game_slug].progress_string()}"
            sleep(3)
    
    def update_button_task(self, game_view_page, button: Gtk.Button):
        initial_game: Game = game_view_page.game

        while game_view_page.game == initial_game:
            button.set_label(self.game_statuses[initial_game.get_slug(True)])
            sleep(3)
        
        print(f"Game {initial_game.name} is over")