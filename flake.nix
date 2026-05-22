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
          ];

          buildInputs = with pkgs; [
            gtk4
            libadwaita
            openssl
          ];

          propagatedBuildInputs = with pkgs; [
            umu-launcher
          ];
        };
      };
    };
}
