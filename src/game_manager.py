from threading import Thread
from time import sleep
from .backend.fitgirl.installer import FitgirlInstaller
from .backend.fitgirl.provider import FitgirlProvider
from .backend.game import Game
from .backend.provider import Installer, Provider

from gi.repository import Gtk, Adw

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
                if installer.downloads[game_slug].progress >= 99:
                    self.game_statuses[game_slug] = "Installing..."
                else:
                    self.game_statuses[game_slug] = f"Downloading... {installer.downloads[game_slug].progress_string()}"
            sleep(3)
    
    def update_button_task(self, game_page, stack: Adw.NavigationView, button: Gtk.Button):
        while True:
            page = stack.get_visible_page()
            if (page != None) and page.get_tag() == "game":
                self.update_button(game_page.game, button)
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
            
            button.connect("clicked", self.get_game_thread, game)
    
    def get_game_thread(self, game):
        Thread(target=self.get_game, args=[game], daemon=True).start()
    
# GameManager singleton
game_manager = GameManager()