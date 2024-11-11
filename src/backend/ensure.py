import os
from subprocess import call

DATA_DIR = "/var/data"

# def first_startup():
#     d = listdir(DATA_DIR)
    
#     if len(d) == 0:
#         call('echo "{}" > ' + f"{DATA_DIR}/metadata.json", shell=True)
#         call(f"touch {DATA_DIR}/igdb.txt", shell=True)
#         call(f"mkdir {DATA_DIR}/pixbufs", shell=True)
#         call(f"mkdir {DATA_DIR}/downloads", shell=True)
#         call(f"mkdir {DATA_DIR}/wine_prefix", shell=True)
#         print("First Startup")
    
#     call(f"WINEPREFIX={DATA_DIR}/wine_prefix wine init", shell=True)

def ensure_file(file_name: str, initial_contents: str | None = None):
    if os.path.exists(f"{DATA_DIR}/{file_name}"): return

    if initial_contents == None:
        call(f"touch {DATA_DIR}/{file_name}", shell=True)
    else:
        call(f'echo "{initial_contents}" > {DATA_DIR}/{file_name}', shell=True)

def ensure_directory(dir_name: str):
    if os.path.exists(f"{DATA_DIR}/{dir_name}"): return
    
    call(f"mkdir {DATA_DIR}/{dir_name}", shell=True)