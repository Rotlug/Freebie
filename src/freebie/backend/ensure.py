import os
from subprocess import call
from .utils import umu_run, DATA_DIR


def ensure_file(file_name: str, initial_contents: str | None = None):
    if os.path.exists(f"{DATA_DIR}/{file_name}"):
        return

    if initial_contents == None:
        call(f"touch {DATA_DIR}/{file_name}", shell=True)
    else:
        call(f'echo "{initial_contents}" > {DATA_DIR}/{file_name}', shell=True)


def ensure_directory(dir_name: str):
    if os.path.exists(f"{DATA_DIR}/{dir_name}"):
        return

    call(f"mkdir {DATA_DIR}/{dir_name}", shell=True)


def find(name, path) -> str | None:
    for root, dirs, files in os.walk(path):
        if name in files:
            return os.path.join(root, name)


def is_non_empty_directory(path: str):
    if not os.path.exists(path) or os.path.isfile(path):
        return False
    if len(os.listdir(path)) > 0:
        return True


def ensure_wine_prefix():
    WINE_PREFIX_PATH = "prefix"
    if is_non_empty_directory(f"{DATA_DIR}/{WINE_PREFIX_PATH}"):
        return
    ensure_directory(WINE_PREFIX_PATH)

    umu_run('""')
