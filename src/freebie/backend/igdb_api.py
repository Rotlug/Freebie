from typing import Any
from .game import Game
from . import utils, json_utils
import requests
import time
from .ensure import ensure_directory, ensure_file
from .utils import DATA_DIR

METADATA_FILE = f"{DATA_DIR}/metadata.json"


class Metadata:
    def __init__(
        self,
        name: str,
        cover_url: str,
        rating: int,
        release_date: str,
        description: str,
    ) -> None:
        self.name = name
        self.cover_url = cover_url
        self.rating = rating
        self.release_date = release_date
        self.description = description

    def debug_print_data(self) -> None:
        print("-----------Cover URL-------------")
        print(self.cover_url)
        print("-----------Rating-------------")
        print(self.rating)
        print("-----------Release Date-------------")
        print(self.release_date)
        print("-----------Description-------------")
        print(self.description)

    @classmethod
    def from_api_data(cls, data: dict[str, Any]):
        return cls(
            cover_url=data["cover"]["url"].lstrip("//").replace("thumb", "720p"),
            description=data.get("summary", ""),
            rating=round(data.get("aggregated_rating", 0)),
            release_date=utils.unix_time_to_string(data.get("first_release_date", 0)),
            name=data.get("name", ""),
        )


# IGDBApiWrapper singleton
class IGDBApiWrapper:
    def __init__(self) -> None:
        self.client_id = ""
        self.secret = ""

        self.access: dict | None = None

        # measure time since access renewal
        self.seconds_left = 0.0
        self.last_renewed = 0.0

        # json cache
        self.cache = json_utils.get_file(METADATA_FILE)

    def generate_access(self) -> None:
        if self.client_id == "" or self.secret == "":
            self.get_credentials()

        self.seconds_left -= time.time() - self.last_renewed

        if (self.access == None) or self.seconds_left <= 50:
            self.access = requests.post(
                f"https://id.twitch.tv/oauth2/token?client_id={self.client_id}&client_secret={self.secret}&grant_type=client_credentials"
            ).json()

            assert type(self.access) == dict
            self.seconds_left = self.access["expires_in"]
            self.last_renewed = time.time()

    def get_credentials(self):
        with open(f"{DATA_DIR}/igdb.txt", "r") as f:
            lines = f.readlines()
            try:
                self.client_id = lines[0].strip()
                self.secret = lines[1].strip()
            except:
                self.client_id = ""
                self.secret = ""

    def update_igdb_credentials_file(self, new_client_id: str, new_secret: str):
        with open(f"{DATA_DIR}/igdb.txt", "w") as f:
            f.write(f"{new_client_id}\n{new_secret}")

    def search(self, game: Game, retry=False) -> Metadata | None:
        slug = game.get_slug(retry)

        # Get From Cache
        if slug in self.cache:
            result = self.cache[slug]
            if result == None:
                if retry:
                    return None
                else:
                    return self.search(game, retry=True)
            return Metadata.from_api_data(result)

        # Get From API
        data = self.fetch_data(
            "https://api.igdb.com/v4/games",
            f'fields cover.url,name,url,summary,aggregated_rating,first_release_date; where slug="{slug}"; limit 1;',
        )

        no_data = data == []

        # NO DATA (no cover also counts as no data)
        if no_data or ("cover" not in data[0]):
            self.cache[slug] = None
            if retry:
                return None  # Give Up
            else:  # Retry with the short version of the slug
                return self.search(game, True)

        # Data Found - Save in cache and continue
        data = data[0]
        self.cache[slug] = data
        return Metadata.from_api_data(data)

    # Helper function to fetch data from igdb api
    def fetch_data(self, endpoint: str, data: str):
        self.generate_access()
        assert self.access is not None

        return requests.post(
            endpoint,
            **{
                "headers": {
                    "Client-ID": self.client_id,
                    "Authorization": f"Bearer {self.access['access_token']}",
                },
                "data": data,
            },
        ).json()

    def save_cache_task(self):
        # Save cache to disk periodically
        while True:
            self.save_cache_to_disk()
            time.sleep(60)

    def save_cache_to_disk(self):
        json_utils.override_file(METADATA_FILE, self.cache)


# IGDB Singleton
ensure_directory("pixbufs")
ensure_file("metadata.json", "{}")
# ensure_file("igdb.txt")
igdb = IGDBApiWrapper()
