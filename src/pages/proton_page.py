from subprocess import call
from threading import Thread
from gi.repository import Adw, Gtk, GLib

import requests
from ..backend.ensure import ensure_directory, ensure_wine_prefix, DATA_DIR, find

URL = "https://github.com/GloriousEggroll/proton-ge-custom/releases/latest"

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
        self.title.set_label("Downloading Proton-GE...")
        r: requests.Response = requests.get(URL)
        assert r.status_code == 200 # FIXME !!!!
        version = r.url.split("/")[-1] #type: ignore

        # Download Proton
        self.title.set_label("Downloading Proton-GE...")
        self.subtitle.set_label("Proton-GE is the tool we use to run Windows games on Linux.")
        download_url = f"https://github.com/GloriousEggroll/proton-ge-custom/releases/download/{version}/{version}.tar.gz"
        ensure_directory("proton")
        self.download_stream(download_url, f"{DATA_DIR}/proton/{version}.tar.gz")
        
        self.title.set_label(f"Installing {version}...")
        call(f"tar -xvzf {DATA_DIR}/proton/{version}.tar.gz", shell=True, cwd=f"{DATA_DIR}/proton/") #Uncompress

        # Download Winetricks
        self.download_winetricks()
        call(f"rm {DATA_DIR}/proton/{version}.tar.gz", shell=True) # Remove the compressed version
        
        ensure_wine_prefix() # make a wine prefix if one doesn't exist

        # Download DXVK
        self.title.set_label("Downloading DXVK...")
        self.subtitle.set_label("DXVK Is the tool we use to emulate DirectX 8-11 on Linux.")
        self.download_dxvk()
        
        # Download VKD3D
        self.title.set_label("Downloading VKD3D...")
        self.subtitle.set_label("VKD3D Is the tool we use to emulate DirectX 12 on Linux.")
        self.download_vkd3d()

        # Bye Bye
        self.nav.pop_to_tag("main")
        
    def download_stream(self, url: str, dest: str):
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

    def download_winetricks(self):
        r: requests.Response = requests.get("https://raw.githubusercontent.com/Winetricks/winetricks/master/src/winetricks")
        with open(f"{DATA_DIR}/proton/winetricks", "wb") as f:
            f.write(r.content) # type: ignore
        call(f"chmod +x {DATA_DIR}/proton/winetricks", shell=True)
    
    def download_dxvk(self):
        r = requests.get("https://github.com/doitsujin/dxvk/releases/latest")
        version = r.url.split("/")[-1].lstrip("v")
        download_url = f"https://github.com/doitsujin/dxvk/releases/download/v{version}/dxvk-{version}.tar.gz"
        
        self.download_stream(download_url, f"{DATA_DIR}/proton/dxvk-{version}.tar.gz")
        call(f"tar -xvzf {DATA_DIR}/proton/dxvk-{version}.tar.gz", shell=True, cwd=f"{DATA_DIR}/proton")
        
        dxvk_dir = f"{DATA_DIR}/proton/dxvk-{version}"
        call(f"cp x64/*.dll {DATA_DIR}/prefix/drive_c/windows/system32", cwd=dxvk_dir, shell=True)
        call(f"cp x32/*.dll {DATA_DIR}/prefix/drive_c/windows/syswow64", cwd=dxvk_dir, shell=True)
    
    def download_vkd3d(self):
        r = requests.get("https://github.com/HansKristian-Work/vkd3d-proton/releases/latest")
        version = r.url.split("/")[-1].lstrip("v")
        download_url = f"https://github.com/HansKristian-Work/vkd3d-proton/releases/download/v{version}/vkd3d-proton-{version}.tar.zst"
        
        self.download_stream(download_url, f"{DATA_DIR}/proton/vkd3d-{version}.tar.zst")
        call(f"tar --zstd -xvf {DATA_DIR}/proton/vkd3d-{version}.tar.zst", shell=True, cwd=f"{DATA_DIR}/proton")
        
        vkd3d_dir = f"{DATA_DIR}/proton/vkd3d-proton-{version}"
        call(f"cp x64/*.dll {DATA_DIR}/prefix/drive_c/windows/system32", cwd=vkd3d_dir, shell=True)
        call(f"cp x32/*.dll {DATA_DIR}/prefix/drive_c/windows/syswow64", cwd=vkd3d_dir, shell=True)