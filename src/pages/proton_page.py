from subprocess import call
from threading import Thread
from gi.repository import Adw, Gtk, GLib
import sys, os

import requests
from ..backend.ensure import ensure_directory, DATA_DIR, ensure_wine_prefix, is_non_empty_directory

@Gtk.Template(resource_path='/com/github/rotlug/Freebie/gtk/proton_page.ui')
class ProtonPage(Adw.NavigationPage):
    __gtype_name__ = 'ProtonPage'
    progressbar: Gtk.ProgressBar = Gtk.Template.Child()
    title : Gtk.Label = Gtk.Template.Child()
    subtitle: Gtk.Label = Gtk.Template.Child()

    def __init__(self, nav: Adw.NavigationView):
        self.nav = nav

        super().__init__()
        Thread(target=self.download, daemon=True).start()

    def download(self):
        ensure_directory("proton")

        # Download UMU-run if doesnt exist
        if (not os.path.exists(f"{DATA_DIR}/proton/umu_run.py")):
            self.title.set_label("Downloadimg UMU...")
            self.subtitle.set_label("UMU Is a tool to run Steam games outside of Steam.")
            self.download_umu()

        # Initialize Wine Prefix, if it doesn't exist
        if not is_non_empty_directory(f"{DATA_DIR}/prefix"):
            self.title.set_label("Creating Wine Prefix...")
            self.subtitle.set_label("This is the folder that the games will be installed in.")
            ensure_wine_prefix()

        # Bye Bye
        self.restart()
        
    def download_stream(self, url: str, dest: str):
        self.progressbar.set_fraction(0)
        r: requests.Response = requests.get(url, stream=True)
        total_size = int(r.headers.get('content-length', 0))
        downloaded_size = 0

        with open(dest, 'wb') as file:
            for chunk in r.iter_content(chunk_size=1024 * 1024): # Download the file in 1MB Chunks
                if chunk:
                    file.write(chunk) # type: ignore
                    downloaded_size += len(chunk)
                    
                    # Calculate and print the progress percentage
                    percent = (downloaded_size / total_size)
                    GLib.idle_add(self.progressbar.set_fraction, percent) 

    def restart(self):
        # Get the current Python executable and script
        python = sys.executable

        # Replace the current process with a new one
        os.execv(python, [python] + sys.argv)
    
    def download_umu(self):
        r = requests.get("https://github.com/Open-Wine-Components/umu-launcher/releases/latest")
        version = r.url.split("/")[-1].lstrip("v")
        download_url = f"https://github.com/Open-Wine-Components/umu-launcher/releases/download/{version}/Zipapp.zip"
        print("DOWNLOAD URL: " + download_url)
        self.download_stream(download_url, f"{DATA_DIR}/proton/umu.zip")
        
        call(f"unzip umu.zip", cwd=f"{DATA_DIR}/proton", shell=True)
        call(f"tar -xf {DATA_DIR}/proton/Zipapp.tar", shell=True, cwd=f"{DATA_DIR}/proton")