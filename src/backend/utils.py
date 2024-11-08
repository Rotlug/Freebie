from datetime import datetime
from unidecode import unidecode

import os

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
