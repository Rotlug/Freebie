from datetime import datetime
from subprocess import call
import shutil

import os
import sys

non_sandboxed_dir = os.path.expanduser("~/.local/share/freebie")
DATA_DIR = os.getenv("XDG_DATA_HOME", non_sandboxed_dir)

# If not running inside Flatpak, make sure to create the directory
if DATA_DIR == non_sandboxed_dir and not os.path.exists(non_sandboxed_dir):
    os.mkdir(non_sandboxed_dir)


def is_in_path(exe: str):
    return shutil.which(exe) is not None


def restart():
    if is_in_path("freebie"):
        os.execvp("freebie", ["freebie"] + sys.argv[1:])
    else:
        # Get the current Python executable and script
        python = sys.executable

        # Replace the current process with a new one
        os.execv(python, [python] + sys.argv)


def any_of_list_in(list_of_str: list[str], check_str: str) -> bool:
    result = False

    for string in list_of_str:
        if string in check_str:
            result = True
            break

    return result


def unix_time_to_string(unix_time: int):
    if unix_time is None:
        return "None"
    return datetime.utcfromtimestamp(unix_time).strftime("%d/%m/%Y")


def split_multiple(string: str, chars: str) -> str:
    new_string = string

    for char in chars:
        new_string = new_string.split(char)[0]

    return new_string


def replace_multiple(old: str, chars: str, repl: str):
    new_string = old

    for char in chars:
        new_string = new_string.replace(char, repl)

    return new_string


def file_dir(file: str):
    return file.rstrip(os.path.basename(file))


def get_absolute_path(file: str, relative: str):
    return os.path.join(file_dir(file), relative)


def quotes_if_space(string: str):
    new_string = string
    if " " in string:
        new_string = f"'{string}'"
    return new_string


def umu_run(exe: str, cwd: str | None = None):
    env = os.environ
    env["GAMEID"] = "0"

    if not env["WINEPREFIX"]:
        env["WINEPREFIX"] = f"{DATA_DIR}/prefix"
    if not env["PROTONPATH"]:
        env["PROTONPATH"] = "GE-Proton"

    print(f"Using proton env variable: {env['PROTONPATH']}")

    python = sys.executable

    if is_in_path("umu-run"):
        call(f"umu-run {exe}", shell=True, env=env, cwd=cwd)
    else:
        call(
            f"{python} {DATA_DIR}/proton/umu/umu_run.py {exe}",
            shell=True,
            env=env,
            cwd=cwd,
        )


def set_wine_sound_driver(sound_driver: str):
    sound_file = f"""Windows Registry Editor Version 5.00

[HKEY_CURRENT_USER\\Software\\Wine\\Drivers]
"Audio"="{sound_driver}"
"""

    with open(f"{DATA_DIR}/sound.reg", "w") as f:
        f.write(sound_file)

    umu_run(f"reg import {DATA_DIR}/sound.reg")


def wrap_in_quotes(string: str):
    return f'"{string.replace('"', '\\"')}"'
