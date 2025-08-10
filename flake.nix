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
    flake-utils.lib.eachDefaultSystem (
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
          cargoHash = "sha256-VeE1kHrc4Q+naTaKLtrh2HA1CAakG43Wq5VvG+2I4iQ=";
          # cargoLock = pkgs.rustPlatform.importCargoLock {
          #   lockFile = ./Cargo.lock;
          # };
        };
        devShells.default = pkgs.mkShell rec {
          packages = (
            with pkgs;
            [
              rust-analyzer
              lldb
              jq
              rustup
              bash
            ]
          );
          inherit buildInputs nativeBuildInputs;

          env = {
            RUST_BACKTRACE = "full";
          };
          shellHook =
            let
              proot = builtins.toString ./.;
              initFile = pkgs.writeText ".bashrc" ''
                echo "Rust shell activated!"
                set -a
                  hw() { echo "Hello world!"; }
                  build() { cargo build; }
                  run() { cargo run; }
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
