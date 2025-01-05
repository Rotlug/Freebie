from .game import Game
from . import utils, json_utils
import requests
import time
from .ensure import ensure_directory, ensure_file

METADATA_FILE = "/var/data/metadata.json"

class Metadata:
    def __init__(self, cover_url, rating, release_date, description) -> None:
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

# IGDBApiWrapper singleton
class IGDBApiWrapper:
    def __init__(self) -> None:
        self.client_id = ""
        self.secret = ""

        self.access: dict | None = None

        # measure time since access renewal
        self.seconds_left = 0
        self.last_renewed = 0
        
        # json cache
        self.cache = json_utils.get_file(METADATA_FILE)

    def generate_access(self) -> None:
        if self.client_id == "" or self.secret == "":
            with open("/var/data/igdb.txt", "r") as f:
                lines = f.readlines()
                self.client_id = lines[0].strip()
                self.secret = lines[1].strip()
            
        self.seconds_left -= (time.time() - self.last_renewed)

        if (self.access == None) or self.seconds_left <= 50:
            self.access = requests.post(f'https://id.twitch.tv/oauth2/token?client_id={self.client_id}&client_secret={self.secret}&grant_type=client_credentials').json()

            assert type(self.access) == dict
            self.seconds_left = self.access["expires_in"]
            self.last_renewed = time.time()

    def search(self, game: Game, retry=False) -> Metadata | None:
        slug = game.get_slug(retry)
        
        # Get From Cache
        if slug in self.cache:
            result = self.cache[slug]
            if result == None:
                if retry: return None
                else: return self.search(game, retry=True)
            return self.dict_to_metadata(result)
        
        # Get From API
        print(f"Fetching {game.name} from api")
        self.generate_access() # Regenerate Access if time is running out
        assert self.access != None
        data = requests.post('https://api.igdb.com/v4/games', **{'headers': {'Client-ID': self.client_id, 'Authorization': f'Bearer {self.access["access_token"]}'},'data': f'fields cover.url,name,url,summary,aggregated_rating,first_release_date; where slug="{slug}"; limit 1;'}).json()

        no_data = (data == [])

        # NO DATA (no cover also counts as no data)
        if no_data or ('cover' not in data[0]):
            self.cache[slug] = None
            if retry:
                return None # Give Up
            else: # Retry with the short version of the slug
                return self.search(game, True)

        # Data Found - Save in cache and continue
        data = data[0]
        self.cache[slug] = data
        return self.dict_to_metadata(data)

    def dict_to_metadata(self, data: dict) -> Metadata:
        if 'first_release_date' not in data: data['first_release_date'] = 0
        if 'aggregated_rating' not in data: data['aggregated_rating'] = 0
        if 'summary' not in data: data['summary'] = ""

        return Metadata(
            cover_url = data['cover']['url'].lstrip('//').replace('thumb', '720p'),
            description = data['summary'],
            rating = round(data['aggregated_rating']),
            release_date=utils.unix_time_to_string(data['first_release_date'])
        )

    def save_cache_task(self):
        # Save cache to disk periodically
        while True:
            self.save_cache_to_disk()
            time.sleep(60)

    def save_cache_to_disk(self):
        print(f"Saved Cache to {METADATA_FILE}")
        json_utils.override_file(METADATA_FILE, self.cache)

# IGDB Singleton
ensure_directory("pixbufs")
ensure_file("metadata.json", "{}")
# ensure_file("igdb.txt")
igdb = IGDBApiWrapper()
