{
  description = "Rust development environment";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs =
    {
      self,
      nixpkgs,
      flake-utils,
    }:
    {
      homeManager = {
        default = ./nix/homeManager.nix;
      };
    }
    // flake-utils.lib.eachDefaultSystem (
      system:
      let
        pkgs = nixpkgs.legacyPackages.${system};
        nativeBuildInputs = [ pkgs.pkg-config ];
        buildInputs = with pkgs; [
          openssl
          alsa-lib
        ];
      in
      {
        packages.default = pkgs.rustPlatform.buildRustPackage {
          inherit buildInputs nativeBuildInputs;
          src = ./.;
          name = "ampterm";
          cargoHash = "sha256-mXG4WJkIF1YHNO0w+WotqHLIqIzNEbjjLAW+dQXkmec=";
        };
        devShells.default = pkgs.mkShell {
          packages = (
            with pkgs;
            [
              lldb
              jq
              rustup
              # This is necessary for opening bash from Neovim
              bash
            ]
          );
          inherit buildInputs nativeBuildInputs;

          env = {
            RUST_BACKTRACE = "full";
          };
          shellHook =
            let
              initFile = pkgs.writeText ".bashrc" ''
                echo "Rust shell activated!"
                set -a
                  hw() { echo "Hello world!"; }
                  build() { nix build; }
                  run() { build; }
                set +a
                # nvim .
              '';
            in
            ''
              bash --init-file ${initFile}; exit
            '';
        };
      }
    );
}
