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
            if " - " in result or " + " in result or " & " in result or ", ":
                result = result.split(" - ")[0]
                result = result.split(" + ")[0]
                result = result.split(" & ")[0]
                result = result.split(", ")[0]
            else:
                result = utils.split_multiple(result, ":–-,")

        result = result.replace("goty", "game of the year")
        result = result.replace("–", "-")
        result = utils.replace_multiple(result, "#$%&'()*+,./:;<=>?@[\\]^_`{|}~!", "")
        result = result.replace(' ', '-').replace("---", "-")
        result = result.replace("--", "-")
        result = result.rstrip('-')
        return result
