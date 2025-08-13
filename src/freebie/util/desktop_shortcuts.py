import os
from subprocess import call

from freebie.backend.game import InstalledGame
from freebie.backend.utils import DATA_DIR, is_in_path, umu_run, wrap_in_quotes


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
TryExec={DesktopShortcuts.get_executable()}
Exec={DesktopShortcuts.get_executable()} --game={wrap_in_quotes(game.name)}
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

    @staticmethod
    def get_executable():
        if is_in_path("freebie"):
            return "freebie"
        return "flatpak run com.github.rotlug.Freebie"
