from datetime import datetime
from subprocess import call

import os

from unidecode import unidecode

DATA_DIR = os.getenv('XDG_DATA_HOME', os.path.expanduser('~/.local/share/freebie'))

# If not running inside Flatpak, make sure to create the directory
if DATA_DIR == os.path.expanduser('~/.local/share/freebie'):
    os.mkdir(os.path.expanduser('~/.local/share/freebie'))

import os, sys

def any_of_list_in(list_of_str: list, check_str: str) -> bool:
    result = False

    for string in list_of_str:
        if string in check_str:
            result = True
            break

    return result


def unix_time_to_string(unix_time: int):
    if unix_time is None:
        return "None"
    return datetime.utcfromtimestamp(unix_time).strftime('%d/%m/%Y')


def split_multiple(string: str, chars: str) -> str:
    new_string = string

    for char in chars:
        new_string = new_string.split(char)[0]

    return new_string


def replace_multiple(old: str, chars: str, repl:str):
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
    if " " in string: new_string = f"'{string}'"
    return new_string

def umu_run(exe: str):
    env = os.environ
    env["GAMEID"] = "0"
    env["WINEPREFIX"] = f"{DATA_DIR}/prefix"
    
    python = sys.executable
        
    call(f'{python} {DATA_DIR}/proton/umu_run.py {exe}', shell=True, env=env)

def set_wine_sound_driver(sound_driver: str):
    sound_file = f"""Windows Registry Editor Version 5.00

[HKEY_CURRENT_USER\\Software\\Wine\\Drivers]
"Audio"="{sound_driver}"
"""
    
    with open(f"{DATA_DIR}/sound.reg", "w") as f:
        f.write(sound_file)
    
    umu_run(f"reg import {DATA_DIR}/sound.reg")
