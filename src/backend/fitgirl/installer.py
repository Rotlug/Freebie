from ..provider import Installer
from ..game import Game
from time import sleep
import aria2p
import subprocess
from os import listdir
from shutil import rmtree

class FitgirlInstaller(Installer):
    def __init__(self) -> None:
        super().__init__()
        subprocess.Popen("aria2c --enable-rpc > /dev/null", shell=True) # Open ARIA2
        self.aria2 = aria2p.API(
            aria2p.Client(
                host="http://localhost",
                port=6800,
                secret=""
            )
        )

    def get_game(self, game: Game) -> None:
        soup = self.get_soup(game.link)

        magnet_link: str = soup.find("a", {"id": "openPopup"})["href"] # type: ignore

        download = self.download(magnet_link, game)
        desktop = self.install(download, game)

        print(f"Game installed at {desktop}")

    # Return path to repack folder
    def download(self, magnet: str, game: Game) -> str:
        self.aria2.add_magnet(magnet, {"dir": "/var/data/downloads/"})

        # Find Download GID
        gid: str = ""
        while gid == "":
            for d in self.aria2.get_downloads():
                if "[METADATA]" in d.name: continue
                name_slug = Game(d.name, "", "").get_slug(True)
                if game.get_slug(True) in name_slug:
                    gid = d.gid
                    break
            sleep(3)

        # Get Progress Percentage Periodically
        while True:
            download = self.aria2.get_download(gid)
            self.downloads[game.get_slug()] = download
            print(download.progress_string(0))
            if download.is_complete or download.seeder:
                download.remove()
                del self.downloads[game.get_slug()] # Remove download from downloads dict

                return download.control_file_path.as_posix().strip('.aria2')
                break # I Am Funi 2
            sleep(3)

    # Returns path to .lnk file
    def install(self, path: str, game: Game) -> str | None:
        winetricks_path = "/home/rotlug/Documents/Code/Game/winetricks" # Placeholder!
        wine_path = "/home/rotlug/Downloads/GE-Proton9-11/files/bin/wine" # Placeholder! 
        wine_prefix = "/home/rotlug/.local/share/Steam/steamapps/compatdata/2508984596/pfx/" # Placeholder!

        # Mute Audio
        subprocess.call(f'WINE="{wine_path}" WINEPREFIX="{wine_prefix}" steam-run "{winetricks_path}" sound=disable', shell=True)

        # Run Installer
        command = f'WINEPREFIX="{wine_prefix}" steam-run "{wine_path}" "{path}/setup.exe" /DIR="C:\\Games\\{game.get_slug()}"'
        subprocess.call(command, shell=True)

        # Turn audio back on
        subprocess.call(f'WINE="{wine_path}" WINEPREFIX="{wine_prefix}" steam-run "{winetricks_path}" sound=pulse', shell=True)

        # Installation is complete, delete repack
        rmtree(path)

        # Look for desktop icon
        desktop_dir = f"{wine_prefix}/drive_c/users/Public/Desktop"
        for d in listdir(desktop_dir):
            slug = Game(d, "", "").get_slug(True)
            if game.get_slug(True) in slug: return desktop_dir + "/" + d

        # No Desktop icon found :(
        return None