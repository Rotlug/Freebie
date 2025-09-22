{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };
  outputs = {
    self,
    nixpkgs,
    flake-utils,
  }:
    flake-utils.lib.eachDefaultSystem
    (
      system: let
        pkgs = import nixpkgs {
          inherit system;
        };
      in
        with pkgs; {
          devShells.default = mkShell {
            nativeBuildInputs = with pkgs; [
              meson
              ninja
              wrapGAppsHook4
              glib
              pkg-config
              appstream
              desktop-file-utils
            ];

            buildInputs =
              [
                flatpak-builder
                blueprint-compiler
                python312
                gtk4
                libadwaita
                cmake
                aria2
                umu-launcher
                icoutils
              ]
              ++ (with pkgs.python312Packages; [
                pygobject-stubs
                pygobject3
                pycairo
                requests
                unidecode
                beautifulsoup4
                types-beautifulsoup4
                aria2p
                pillow
              ]);

            shellHook = "
              XDG_DATA_DIRS=$XDG_DATA_DIRS:$GSETTINGS_SCHEMAS_PATH:${hicolor-icon-theme}/share:${adwaita-icon-theme}/share
            ";
          };
        }
    );
}
