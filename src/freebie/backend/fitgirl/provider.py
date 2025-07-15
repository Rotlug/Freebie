from bs4 import Tag
from .installer import FitgirlInstaller
from ..provider import Provider
from ..game import Game
from .. import utils


class FitgirlProvider(Provider):
    def __init__(self) -> None:
        super().__init__()

    def get_popular(self) -> list[Game]:
        return self.search("Fitgirl")

    def fitgirl_site_search(self, query: str):
        soup = self.get_soup(f"https://fitgirl-repacks.site/?s={query}")
        games: list[Game] = []

        for tag in soup.find_all("article", {"class": "category-lossless-repack"}):
            tag: Tag

            title_tag: Tag = tag.find("h1", {"class": "entry-title"})  # type: ignore
            title = title_tag.get_text()

            link_tag = title_tag.find("a")  # type: ignore
            link: str = link_tag["href"]  # type: ignore

            size_p: str = (
                (
                    tag.find("p")
                    .get_text()  # type: ignore
                    .lower()
                    .split("original size:")[1]
                    .split("b")[0]
                    + "b"
                )
                .upper()
                .strip()
            )

            game = Game(name=title, link=link, size=size_p)
            game.installer = FitgirlInstaller

            games.append(game)

        return games

    def search(self, query) -> list[Game]:
        if query == "Fitgirl":
            return self.leetx_search(query)

        return self.fitgirl_site_search(query)

    # Search in 1337x (to find popular games that are not porn -_-)
    def leetx_search(self, query: str, page=1) -> list[Game]:
        soup = self.get_soup(f"https://1337x.to/category-search/{query}/Games/{page}/")
        games: list[Game] = []

        game_names = set()  # No Duplicate names allowed
        for tag in soup.find_all("td", {"class": "coll-1"}):
            children = list(tag.children)
            parent = list(tag.parents)[0]

            uploader = parent.findChild("td", {"class": "coll-5"}).text
            if uploader != "FitGirl":
                continue

            download_size = (
                parent.findChild("td", {"class": "coll-4"}).text.split("B")[0] + "B"
            )

            entry_name = children[1].contents[0]
            game_link = "https://1337x.to" + children[1]["href"]

            game_name = utils.split_multiple(
                entry_name, "([/"
            ).strip()  # FitGirl Seperator
            if game_name in game_names:
                continue
            game_names.add(game_name)

            # Shorten name if its really long
            game_short_name = utils.split_multiple(game_name, ":+/–-").rstrip()
            if len(game_name) > 50:
                game_name = game_short_name

            game = Game(
                name=game_name,
                link=game_link,
                size=download_size,
            )
            game.installer = FitgirlInstaller

            games.append(game)

        return games
