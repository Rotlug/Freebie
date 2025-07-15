import os
from subprocess import call
from threading import Thread
import time
from typing import TYPE_CHECKING

from .backend.utils import is_in_path, umu_run, wrap_in_quotes
from .backend.fitgirl.installer import FitgirlInstaller
from .backend.fitgirl.provider import FitgirlProvider
from .backend.game import Game, InstalledGame
from .backend.provider import Installer, Provider

from .backend.ensure import ensure_directory, DATA_DIR

from .backend import json_utils

from gi.repository import Gtk, Adw, GLib

from datetime import timedelta

if TYPE_CHECKING:
    from freebie.pages.game_page import GamePage
    from freebie.pages.play_view import PlayView


class Source:
    def __init__(self, provider: type[Provider], installer: type[Installer]):
        self.provider = provider()
        self.installer = installer()


class GameManager:
    def __init__(self) -> None:
        self.sources = [Source(FitgirlProvider, FitgirlInstaller)]

        self.game_statuses: dict[str, str] = {}
        self.play_view: PlayView | None = None
        self.game_page: GamePage | None = None

    def search(self, query: str) -> list[Game]:
        if query == "":
            return self.get_popular()

        games: list[Game] = []
        for source in self.sources:
            games += source.provider.search(query)

        return games

    def get_popular(self) -> list[Game]:
        games = []
        for source in self.sources:
            games += source.provider.get_popular()

        return games

    def add_custom_game_to_installed(self, game: InstalledGame):
        json_utils.add_to_file(f"{DATA_DIR}/installed.json", game.name, game.to_dict())

        if self.play_view is not None:
            self.play_view.update_game_array()

    def get_game(self, game: Game):
        installer = game.installer()
        installer_thread = Thread(target=installer.get_game, args=[game], daemon=True)
        installer_thread.start()

        game_slug = game.get_slug(True)

        self.game_statuses[game_slug] = "Downloading... 0%"
        while installer_thread.is_alive():
            if game_slug in installer.downloads:
                self.game_statuses[game_slug] = (
                    f"Downloading... {installer.downloads[game_slug].progress_string()}"
                )
            else:
                self.game_statuses[game_slug] = "Installing..."
            time.sleep(3)

        else:
            del self.game_statuses[game_slug]

        if self.play_view is not None:
            self.play_view.update_game_array()
        if self.game_page is not None:
            installed_game = self.is_installed(game)
            if installed_game:
                self.game_page.set_game(installed_game)
            else:
                self.game_page.set_game(game)

    def update_button_task(self, game_page, nav: Adw.NavigationView):
        while True:
            page = nav.get_visible_page()
            if (page is not None) and page.get_tag() == "game":
                GLib.idle_add(
                    self.update_button, game_page.game, game_page.action_button
                )
            time.sleep(3)

    def update_button(self, game: Game, button: Gtk.Button):
        slug = game.get_slug(True)
        if slug in self.game_statuses:
            button.set_sensitive(False)
            button.set_css_classes([])
            button.remove_css_class("suggested-action")
            button.set_label(self.game_statuses[slug])
        else:
            if game.name in json_utils.get_file(f"{DATA_DIR}/installed.json"):
                button.set_sensitive(True)
                button.set_label("Play")
                button.add_css_class("suggested-action")
            else:
                button.set_sensitive(True)
                button.set_label(f"Get ({game.size})")
                button.add_css_class("suggested-action")

    def get_game_thread(self, game: Game):
        Thread(target=self.get_game, args=[game], daemon=True).start()

    def run_game_thread(self, game: Game):
        Thread(target=self.run_game, args=[game], daemon=True).start()

    def run_game(self, game: Game):
        self.game_statuses[game.get_slug(True)] = "Running"
        installed = json_utils.get_file(f"{DATA_DIR}/installed.json")
        if game.name not in installed:
            return

        installed_game = InstalledGame.from_dict(game.name, installed[game.name])

        start_time = time.time()  # Track number of seconds that game has been open

        umu_run(f'"{installed_game.exe}"')

        seconds_elapsed = int(time.time() - start_time)

        installed_game.seconds_played += seconds_elapsed

        json_utils.add_to_file(
            f"{DATA_DIR}/installed.json", installed_game.name, installed_game.to_dict()
        )

        if self.game_page is not None:
            installed_game.metadata
            self.game_page.set_game(installed_game)  # Update with current time played
        if self.play_view is not None:
            self.play_view.update_game_array()

        del self.game_statuses[
            game.get_slug(True)
        ]  # Remove "Running" status from the game

    def get_button_target(self, game: Game):
        if self.is_installed(game):
            return self.run_game_thread
        return self.get_game_thread

    def uninstall(self, game: InstalledGame):
        DesktopShortcuts.remove(game)

        call(
            f"rm -r {json_utils.get_file(f'{DATA_DIR}/installed.json')[game.name]['dir']}",
            shell=True,
        )
        json_utils.remove_from_file(f"{DATA_DIR}/installed.json", game.name)
        if self.play_view is not None:
            self.play_view.update_game_array()

    def is_installed(self, game: Game) -> InstalledGame | None:
        installed = json_utils.get_file(f"{DATA_DIR}/installed.json")
        if game.name in installed:
            return InstalledGame.from_dict(game.name, installed[game.name])

        return None

    def format_duration(self, seconds: int) -> str:
        delta = timedelta(seconds=seconds)
        days = delta.days
        hours, remainder = divmod(delta.seconds, 3600)
        minutes, seconds = divmod(remainder, 60)

        parts = []
        if days:
            parts.append(f"{days} day{'s' if days != 1 else ''}")
        if hours:
            parts.append(f"{hours} hour{'s' if hours != 1 else ''}")
        if minutes:
            parts.append(f"{minutes} minute{'s' if minutes != 1 else ''}")
        if seconds:
            parts.append(f"{seconds} second{'s' if seconds != 1 else ''}")

        return (
            ", ".join(parts[:-1])
            + (" and " + parts[-1] if len(parts) > 1 else parts[0])
            if parts
            else "0 seconds"
        )

    def get_all_installed_games(self):
        installed = json_utils.get_file(f"{DATA_DIR}/installed.json")

        games: list[InstalledGame] = []
        for g in installed.keys():
            name: str = g

            obj = installed[g]

            exe: str = obj.get("exe", "")
            directory: str = obj.get("dir", "")
            seconds_played: int = obj.get("seconds_played", 0)

            game = InstalledGame(name, exe, directory)
            game.seconds_played = seconds_played

            games.append(game)

        return games


class DesktopShortcuts:
    @staticmethod
    def create(game: InstalledGame):
        icon_location = f"{DATA_DIR}/icons/{game.get_slug(True)}_icon.png"

        if game.exe.endswith(".lnk"):
            command = f"winemenubuilder -t {wrap_in_quotes(game.exe)} {icon_location}"
            # Generate Icon using winemenubuilder
            umu_run(command)
        elif game.exe.endswith(".exe") and is_in_path("wrestool"):
            command = f"wrestool -x -t14 --output={wrap_in_quotes(icon_location)} {wrap_in_quotes(game.exe)}"
            call(command, shell=True)

        desktop_shortcut = f"""
[Desktop Entry]
Type=Application
Name={game.name}
Comment=
Icon={icon_location}
TryExec={get_executable()}
Exec={get_executable()} --game={wrap_in_quotes(game.name)}
Categories=Game;
StartupNotify=true
Terminal=false
""".strip()

        # Create desktop shortcut
        for path in DesktopShortcuts._get_paths(game):
            with open(path, "w") as f:
                f.write(desktop_shortcut)

            call(f"chmod +x {wrap_in_quotes(path)}", shell=True)

    @staticmethod
    def _get_paths(game: InstalledGame):
        return [
            os.path.expanduser(f"~/Desktop/{game.name}.desktop"),  # ~/Desktop
            os.path.expanduser(
                f"~/.local/share/applications/{game.name}.desktop"
            ),  # ~/.local/share/applications
        ]

    @staticmethod
    def remove(game: InstalledGame):
        for path in DesktopShortcuts._get_paths(game):
            if os.path.exists(path):
                os.remove(path)


def get_executable():
    if is_in_path("freebie"):
        return "freebie"
    return "flatpak run com.github.rotlug.Freebie"


# GameManager singleton
ensure_directory("downloads")
ensure_directory("icons")
game_manager = GameManager()
