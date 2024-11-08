from .installer import FitgirlInstaller
from ..provider import Provider
from ..game import Game
from ..igdb_api import IGDBApiWrapper
from .. import utils

class FitgirlProvider(Provider):
    def __init__(self) -> None:
        super().__init__()
        self.igdb = IGDBApiWrapper()

    def get_popular(self) -> list[Game]:
        return self.search("Fitgirl")

    def search(self, query: str, page=1) -> list[Game]:
        soup = self.get_soup(f"https://1337x.to/category-search/{query}/Games/{page}/")
        games: list[Game] = []

        game_names = set() # No Duplicate names allowed
        for tag in soup.find_all("td", {"class": "coll-1"}):
            children = list(tag.children)
            parent = list(tag.parents)[0]

            uploader = parent.findChild("td", {"class": "coll-5"}).text
            if uploader != "FitGirl": continue

            download_size = parent.findChild("td", {"class": "coll-4"}).text.split("B")[0] + "B"

            entry_name = children[1].contents[0]
            game_link = "https://1337x.to" + children[1]["href"]

            game_name = utils.split_multiple(entry_name, "([/").strip() # FitGirl Seperator
            # if game_name in game_names: continue
            game_names.add(game_name)

            # Shorten name if its really long
            game_short_name = utils.split_multiple(game_name, ":+/–-").rstrip()
            if len(game_name) > 50: game_name = game_short_name

            game = Game(
                name=game_name,
                link=game_link,
                size=download_size,
            )

            games.append(game)

        for game in games:
            game.installer = FitgirlInstaller
            game.metadata = self.igdb.search(game)

        return games
