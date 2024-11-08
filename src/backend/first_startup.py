from os import listdir
from subprocess import call

DATA_DIR = "/var/data"

def first_startup():
    d = listdir(DATA_DIR)
    
    if len(d) == 0:
        call('echo "{}" > ' + f"{DATA_DIR}/metadata.json", shell=True)
        call(f"touch {DATA_DIR}/igdb.txt", shell=True)
        call(f"mkdir {DATA_DIR}/pixbufs", shell=True)
        call(f"mkdir {DATA_DIR}/downloads", shell=True)
        print("First Startup")