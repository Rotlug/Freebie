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

      perSystem = {
        pkgs,
        config,
        ...
      }: let
        cargoToml = fromTOML (builtins.readFile ./Cargo.toml);
      in {
        packages.default = pkgs.rustPlatform.buildRustPackage {
          pname = cargoToml.package.name;
          version = cargoToml.package.version;

          src = ./.;
          cargoHash = "sha256-ZPtfYZ7aaMqCS9uXG0VIIkjxK2qyiYcrg8Dio7ft510=";

          nativeBuildInputs = with pkgs; [
            pkg-config
            wrapGAppsHook4
            makeWrapper
          ];

          buildInputs = with pkgs; [
            gtk4
            libadwaita
            openssl
            libseccomp
            glycin-loaders
            bubblewrap
            umu-launcher
            icoutils
          ];

          postInstall = ''
            wrapProgram $out/bin/${cargoToml.package.name} \
              --prefix PATH : ${pkgs.lib.makeBinPath [pkgs.umu-launcher pkgs.icoutils]}
          '';
        };

        devShells.default = pkgs.mkShell {
          inputsFrom = [config.packages.default];

          nativeBuildInputs = with pkgs; [
            cargo
            rustc
            rustPlatform.bindgenHook
          ];

          shellHook = ''
            export XDG_DATA_DIRS="${pkgs.glycin-loaders}/share:${pkgs.gsettings-desktop-schemas}/share:${pkgs.gtk4}/share/gsettings-schemas/${pkgs.gtk4.name}:$XDG_DATA_DIRS"
          '';
        };
      };
    };
}
