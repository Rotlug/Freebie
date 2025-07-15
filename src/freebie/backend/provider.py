from aria2p import Download
import requests
from .game import Game, InstalledGame
from bs4 import BeautifulSoup as bs
from . import json_utils
from .ensure import ensure_file, DATA_DIR


# Provide `Game` Objects
class Provider:
    def __init__(self) -> None:
        pass

    def search(self, query: str) -> list[Game]:
        return []

    def get_soup(self, url: str) -> bs:
        user_agent = "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/36.0.1985.143 Safari/537.36"
        headers = {"User-Agent": user_agent}

        request_text = requests.get(
            url=url, headers=headers
        ).text  # returns in 'iso-8859-1' (bad)
        try:
            request_text = bytes(
                request_text, "iso-8859-1"
            ).decode(
                "utf-8"
            )  # turns it to 'utf-8' (good, supports diacritics for example the é in Pokémon)
        except Exception:
            pass

        soup = bs(request_text, "html.parser")
        return soup

    def get_popular(self) -> list[Game]:
        return []


# Download and install `Game` objects (it inherits from provider only to get access to the `get_soup` method)
class Installer(Provider):
    def __init__(self) -> None:
        ensure_file("installed.json", initial_contents="{}")
        self.downloads: dict[str, Download] = {}

    # # Download a game and return the path to the downloaded folder
    # def download(self, game: Game) -> str | None:
    #     pass

    # # Install the game and return the path to the exe
    # def install(self, download_folder: str, game: Game) -> str | None:
    #     pass

    # Download and install a game and return the path to the .lnk file of the game
    def get_game(self, game: Game) -> str | None:
        pass

    def add_game_to_installed(self, installed_game: InstalledGame):
        json_utils.add_to_file(
            f"{DATA_DIR}/installed.json", installed_game.name, installed_game.to_dict()
        )
