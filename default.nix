{
  lib,
  appstream,
  blueprint-compiler,
  desktop-file-utils,
  fetchFromGitHub,
  umu-launcher,
  aria2,
  glib,
  gobject-introspection,
  gtk4,
  libadwaita,
  icoutils,
  meson,
  ninja,
  nix-update-script,
  pkg-config,
  python3Packages,
  wrapGAppsHook4,
  lsb-release,
  pciutils,
  procps,
}:

python3Packages.buildPythonApplication {
  pname = "freebie";
  version = "0.1";
  pyproject = false;

  src = fetchFromGitHub {
    owner = "Rotlug";
    repo = "Freebie";
    rev = "83644147f8c14824edad417aa341354bd9ce2324";
    hash = "sha256-swkmuZnD3LCf6NgrVFhO9iNgCTh9iIf3SV713yh20lk=";
  };

  strictDeps = true;

  nativeBuildInputs = [
    appstream
    blueprint-compiler
    desktop-file-utils # for `desktop-file-validate`
    glib # for `glib-compile-schemas`
    gobject-introspection
    gtk4 # for `gtk-update-icon-cache`
    meson
    ninja
    pkg-config
    wrapGAppsHook4
  ];

  buildInputs = [
      libadwaita

      # Undocumented (subprocess.Popen())
      lsb-release
      pciutils
      procps

      aria2
  ];

  propagatedNativeBuildInputs = [
    aria2
    umu-launcher
    icoutils
  ];

  dependencies = with python3Packages; [ 
  	pygobject3
	pycairo
	requests
	unidecode
	beautifulsoup4
	aria2p
	pillow
];

  dontWrapGApps = true;
  makeWrapperArgs = [ "\${gappsWrapperArgs[@]}" "--prefix PATH : ${lib.makeBinPath [ aria2 umu-launcher ]}" ];

  # NOTE: `postCheck` is intentionally not used here, as the entire checkPhase
  # is skipped by `buildPythonApplication`
  # https://github.com/NixOS/nixpkgs/blob/9d4343b7b27a3e6f08fc22ead568233ff24bbbde/pkgs/development/interpreters/python/mk-python-derivation.nix#L296
  #postInstallCheck = ''
   # mesonCheckPhase
  #'';

  passthru = {
    updateScript = nix-update-script { };
  };

  meta = {
    description = "test";
    homepage = "https://github.com/Rotlug/Freebie";
    license = lib.licenses.gpl3Plus;
    mainProgram = "freebie";
    platforms = lib.platforms.linux;
  };
}
