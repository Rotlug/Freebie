# Work with json files to make the app fazter
import json
import os

def get_file(filename: str) -> dict:    
    with open(filename) as file:
        return json.load(file)

def is_in_file(filename, key) -> bool:
    with open(filename) as file:
        json_file = json.load(file)
        if key in json_file: return True
        return False

def override_file(filename, data: dict) -> None:
    with open(filename, "w") as f:
        json.dump(data, f)

def add_to_file(filename: str, key: str, value: str) -> None:
    if is_in_file(filename, key):
        return

    new_file = get_file(filename)
    new_file[key] = value

    override_file(filename, new_file)

def remove_from_file(filename, key: str) -> None:
    file = get_file(filename)
    del file[key]

    override_file(filename, file)