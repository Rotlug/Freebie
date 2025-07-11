from . import utils
from unidecode import unidecode
from typing import TYPE_CHECKING

if TYPE_CHECKING:
    from igdb_api import Metadata
    from .provider import Installer

class Game:
    def __init__(self, name: str, link: str, size: str) -> None:
        self.name = name
        self.link = link
        self.size = size
        self.metadata: Metadata | None = None
        self.installer: type[Installer]
    
    def get_slug(self, short=False):
        result = unidecode(self.name).lower()
        # [short] removes editions, deluxe editions etc..
        if short:
            result = remove_editions(result)
            if "-" in result or "+" in result or "&" in result or ",":
                result = utils.split_multiple(result, "/,-+&")
            else:
                result = utils.split_multiple(result, ":–-,/")

        result = result.replace("goty", "game of the year")
        result = result.replace("–", "-")
        result = utils.replace_multiple(result, "#$%&'()*+,./:;<=>?@[\\]^_`{|}~!", "")
        result = result.replace(' ', '-').replace("---", "-")
        result = result.replace("--", "-")
        result = result.rstrip('-')
        return result.strip()

class InstalledGame(Game):
    def __init__(self, name: str, exe: str, directory: str) -> None:
        super().__init__(name, "", "")

        self.exe = exe
        self.directory = directory
        self.seconds_played = 0

    def to_dict(self):
        return {
                "exe": self.exe,
                "dir": self.directory,
                "seconds_played": self.seconds_played
            }

    @classmethod
    def from_dict(cls, name: str, data: dict[str, str]):
        installed_game = cls(
            name=name,
            exe=data.get("exe", ""),
            directory=data.get("dir", "")
        )

        installed_game.seconds_played = data.get("seconds_played", 0)

        return installed_game

def remove_editions(slug: str):
    editions = [
        "digital deluxe",
        "deluxe",
        "premium",
        "the one who waits bundle"
    ]

    for ed in editions:
        if ed in slug.lower():
            slug = slug.split(ed)[0]
            break
    
    return slug
