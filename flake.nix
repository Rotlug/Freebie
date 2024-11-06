{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };
  outputs = { self, nixpkgs, flake-utils }:
    flake-utils.lib.eachDefaultSystem
      (system:
        let
          pkgs = import nixpkgs {
            inherit system;
          };
        in
        with pkgs;
        {
          devShells.default = mkShell {
            buildInputs = [
              gtk4
              libadwaita
              flatpak-builder
              meson
              ninja

              python312
              python312Packages.pygobject-stubs
              python312Packages.pygobject3
              python312Packages.pycairo
            ];
          };
        }
      );
}
