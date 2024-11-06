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
              flatpak-builder
              meson
              ninja

              python312
              blueprint-compiler
            ] ++ (with pkgs.python312Packages; [
              pygobject-stubs
              pygobject3
              pycairo
              requests
              unidecode
              beautifulsoup4
              types-beautifulsoup4
              aria2p
            ]);
          };
        }
      );
}
