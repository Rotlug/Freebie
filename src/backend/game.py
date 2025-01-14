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

def remove_editions(slug: str):
    editions = [
        "digital deluxe",
        "deluxe",
        "premium",
        "the one who waits bundle"
    ]

    for ed in editions:
        if ed in slug.lower():
            print(f"FOUND {ed} in {slug}")
            slug = slug.split(ed)[0]
            break
    
    return slug
