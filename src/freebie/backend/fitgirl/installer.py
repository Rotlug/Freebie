from ...backend.utils import set_wine_sound_driver, umu_run
from ..provider import Installer
from ..game import Game, InstalledGame
from time import sleep
import aria2p
import subprocess
from os import listdir
from shutil import rmtree
from ..ensure import DATA_DIR

import atexit

proc = subprocess.Popen(["aria2c", "--enable-rpc"], stdout=subprocess.DEVNULL)

aria2 = aria2p.API(aria2p.Client(host="http://localhost", port=6800, secret=""))


class FitgirlInstaller(Installer):
    def __init__(self) -> None:
        super().__init__()
        atexit.register(self.cleanup)

    def cleanup(self):
        print("Stopping aria2")
        proc.kill()

    def get_game(self, game: Game) -> None:
        print(f"GAME_LINK {game.link}")
        soup = self.get_soup(game.link)

        magnet_link = ""

        try:  # 1337x magnet link
            magnet_link: str = soup.find("a", {"id": "openPopup"})["href"]  # type: ignore
        except Exception:
            magnet_link: str = soup.find(
                lambda tag: tag.name == "a" and tag.text == "magnet"
            )["href"]  # type: ignore

        print(f"MAGNET_LINK: {magnet_link}")

        download = self.download(magnet_link, game)
        desktop = self.install(download, game)

        print(f"Game installed at {desktop}")

    # Return path to repack folder
    def download(self, magnet: str, game: Game) -> str:
        aria2.add_magnet(magnet, {"dir": f"{DATA_DIR}/downloads/"})

        # Find Download GID
        gid: str = ""
        super_short_game_slug = game.get_slug(True)[0:10]
        while gid == "":
            for d in aria2.get_downloads():
                if "[METADATA]" in d.name:
                    continue
                name_slug = Game(d.name, "", "").get_slug(True)
                if super_short_game_slug in name_slug:
                    gid = d.gid
                    break
            sleep(3)

        # Get Progress Percentage Periodically
        while True:
            download = aria2.get_download(gid)
            self.downloads[game.get_slug()] = download
            print(download.progress_string(0))
            if download.is_complete or download.seeder:
                sleep(10)
                download.remove()
                del self.downloads[
                    game.get_slug()
                ]  # Remove download from downloads dict

                return download.control_file_path.as_posix().strip(".aria2")
            sleep(3)

    # Returns path to .lnk file
    def install(self, path: str, game: Game) -> str | None:
        wine_prefix = f"{DATA_DIR}/prefix"

        # Mute Audio
        set_wine_sound_driver("disabled")

        # Run Installer
        umu_run(f'"{path}/setup.exe" /VERYSILENT /DIR="C:\\Games\\{game.get_slug()}"')

        # Turn audio back on
        set_wine_sound_driver("pulse")

        # Installation is complete, delete repack
        rmtree(path)

        # Look for desktop icon
        desktop_dir = f"{wine_prefix}/drive_c/users/Public/Desktop"
        game_super_short_slug = game.get_slug(True)[0:5]

        for d in listdir(desktop_dir):
            slug = Game(d, "", "").get_slug(True)
            if game_super_short_slug in slug:
                lnk_path = desktop_dir + "/" + d

                installed_game = InstalledGame(
                    name=game.name,
                    exe=lnk_path,
                    directory=f"{DATA_DIR}/prefix/drive_c/Games/{game.get_slug()}",
                )
                self.add_game_to_installed(installed_game)
                return lnk_path

        # No Desktop icon found :(
        return None
