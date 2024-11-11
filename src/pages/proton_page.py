from subprocess import call
from threading import Thread
from gi.repository import Adw, Gtk, GLib

import requests
from ..backend.ensure import ensure_directory, ensure_wine_prefix, DATA_DIR

URL = "https://github.com/GloriousEggroll/proton-ge-custom/releases/latest"

@Gtk.Template(resource_path='/com/github/rotlug/Freebie/gtk/proton_page.ui')
class ProtonPage(Adw.NavigationPage):
    __gtype_name__ = 'ProtonPage'
    progressbar: Gtk.ProgressBar = Gtk.Template.Child()
    title : Gtk.Label = Gtk.Template.Child()

    def __init__(self, nav: Adw.NavigationView):
        self.nav = nav

        super().__init__()
        Thread(target=self.download, daemon=True).start()

    def download(self):
        self.title.set_label("Downloading Proton-GE...")
        r: requests.Response = requests.get(URL)
        assert r.status_code == 200 # FIXME !!!!
        version = r.url.split("/")[-1] #type: ignore

        download_url = f"https://github.com/GloriousEggroll/proton-ge-custom/releases/download/{version}/{version}.tar.gz"

        download_request: requests.Response = requests.get(download_url, stream=True)
        total_size = int(download_request.headers.get('content-length', 0))
        downloaded_size = 0

        ensure_directory("proton")

        with open(f"{DATA_DIR}/proton/{version}.tar.gz", 'wb') as file:
            for chunk in download_request.iter_content(chunk_size=1024 * 1024): # Download the file in 1MB Chunks
                if chunk:
                    file.write(chunk) # type: ignore
                    downloaded_size += len(chunk)
                    
                    # Calculate and print the progress percentage
                    percent = (downloaded_size / total_size)
                    GLib.idle_add(self.progressbar.set_fraction, percent) 

            self.title.set_label(f"Installing {version}...")
            call(f"tar -xvzf {DATA_DIR}/proton/{version}.tar.gz", shell=True, cwd=f"{DATA_DIR}/proton/") #Uncompress

        self.download_winetricks()

        call(f"rm {DATA_DIR}/proton/{version}.tar.gz", shell=True) # Remove the compressed version
        
        ensure_wine_prefix() # make a wine prefix if one doesn't exist

        self.nav.pop_to_tag("main")

    def download_winetricks(self):
        r: requests.Response = requests.get("https://raw.githubusercontent.com/Winetricks/winetricks/master/src/winetricks")
        with open(f"{DATA_DIR}/proton/winetricks", "wb") as f:
            f.write(r.content) # type: ignore
        call(f"chmod +x {DATA_DIR}/proton/winetricks", shell=True)