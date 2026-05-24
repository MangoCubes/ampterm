{
  description = "Rust development environment";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay.url = "github:oxalica/rust-overlay";
  };

  outputs =
    {
      self,
      nixpkgs,
      flake-utils,
      rust-overlay,
    }:
    {
      homeManager = {
        default = ./nix/homeManager.nix;
      };
    }
    // flake-utils.lib.eachDefaultSystem (
      system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs {
          inherit system overlays;
        };
        rustToolchain = pkgs.rust-bin.stable.latest.default.override {
          extensions = [
            "rust-src"
            "rust-analyzer"
          ];
        };
        nativeBuildInputs = [ pkgs.pkg-config ];
        buildInputs =
          (with pkgs; [
            openssl
            alsa-lib
          ])
          ++ [ rustToolchain ];
      in
      {
        packages.default = pkgs.rustPlatform.buildRustPackage {
          inherit buildInputs nativeBuildInputs;
          src = ./.;
          name = "ampterm";
          cargoHash = "sha256-njAVC9ha6Lp8CsrlpCp635PVyMCU0kOHEnkqdZpKUhQ=";
          doCheck = false;
        };
        meta = {
          description = "OpenSubsonic compatible, keyboard oriented terminal music player";
          homepage = "https://github.com/MangoCubes/ampterm";
          license = nixpkgs.lib.licenses.gplv3;
          maintainers = [ ];
        };
        devShells.default = pkgs.mkShell {
          packages = (
            with pkgs;
            [
              lldb
              jq
              perf # For permformance checking
              # This is necessary for opening bash from Neovim
              bashInteractive
            ]
          );
          inherit buildInputs nativeBuildInputs;

          env = {
            RUST_BACKTRACE = "full";
            RUST_SRC_PATH = "${rustToolchain}/lib/rustlib/src/rust/library";
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
