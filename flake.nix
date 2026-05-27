{
  inputs = {
    flake-parts.url = "github:hercules-ci/flake-parts";
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
  };

  outputs = inputs @ {flake-parts, ...}:
    flake-parts.lib.mkFlake {inherit inputs;} {
      systems = [
        "x86_64-linux"
        "aarch64-linux"
      ];

      perSystem = {pkgs, ...}: {
        devShells.default = pkgs.mkShell {
          nativeBuildInputs = with pkgs; [
            cargo
            rustc
            rustPlatform.bindgenHook
            pkg-config
            wrapGAppsHook4
          ];

          buildInputs = with pkgs; [
            gtk4
            libadwaita
            openssl
            libseccomp
            glycin-loaders
            bubblewrap
          ];

          propagatedBuildInputs = with pkgs; [
            umu-launcher
            icoutils
          ];

          shellHook = ''
            export XDG_DATA_DIRS="${pkgs.glycin-loaders}/share:${pkgs.gsettings-desktop-schemas}/share:${pkgs.gtk4}/share/gsettings-schemas/${pkgs.gtk4.name}:$XDG_DATA_DIRS"
          '';
        };
      };
    };
}
