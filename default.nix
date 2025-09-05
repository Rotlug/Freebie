{
  lib,
  appstream,
  blueprint-compiler,
  desktop-file-utils,
  fetchfromgithub,
  umu-launcher,
  aria2,
  glib,
  gobject-introspection,
  gtk4,
  libadwaita,
  meson,
  ninja,
  nix-update-script,
  pkg-config,
  python3packages,
  icoutils,
  wrapgappshook4,
  lsb-release,
  pciutils,
  procps,
}:
python3packages.buildpythonapplication {
  pname = "freebie";
  version = "0.1";
  pyproject = false;

  src = fetchfromgithub {
    owner = "rotlug";
    repo = "freebie";
    rev = "8bc872449bd7224bf43b5c6ddd8f0cb3197bdd1a";
    hash = "sha256-cv2wq7zxbfd+tywcma214rkugoh9c3etfmiyy6q1j6s=";
  };

  strictdeps = true;

  nativebuildinputs = [
    appstream
    blueprint-compiler
    desktop-file-utils # for `desktop-file-validate`
    glib # for `glib-compile-schemas`
    gobject-introspection
    gtk4 # for `gtk-update-icon-cache`
    meson
    ninja
    pkg-config
    wrapgappshook4
  ];

  buildinputs = [
    libadwaita

    # undocumented (subprocess.popen())
    lsb-release
    pciutils
    procps

    aria2
  ];

  propagatednativebuildinputs = [
    aria2
    umu-launcher
    icoutils
  ];

  dependencies = with python3packages; [
    pygobject3
    pycairo
    requests
    unidecode
    beautifulsoup4
    aria2p
    pillow
  ];

  dontwrapgapps = true;
  makewrapperargs = ["\${gappswrapperargs[@]}" "--prefix path : ${lib.makebinpath [aria2 umu-launcher]}"];

  # note: `postcheck` is intentionally not used here, as the entire checkphase
  # is skipped by `buildpythonapplication`
  # https://github.com/nixos/nixpkgs/blob/9d4343b7b27a3e6f08fc22ead568233ff24bbbde/pkgs/development/interpreters/python/mk-python-derivation.nix#l296
  #postinstallcheck = ''
  # mesoncheckphase
  #'';

  passthru = {
    updatescript = nix-update-script {};
  };

  meta = {
    description = "test";
    homepage = "https://github.com/rotlug/freebie";
    license = lib.licenses.gpl3plus;
    mainprogram = "freebie";
    platforms = lib.platforms.linux;
  };
}
