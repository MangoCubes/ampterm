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
    let
      rust-toolchain = ''
        [toolchain]
        channel = "stable"
      '';
    in
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        pkgs = nixpkgs.legacyPackages.${system};
        # Read the file relative to the flake's root
        # overrides = (builtins.fromTOML (builtins.readFile (self + "/rust-toolchain.toml")));
        overrides = (builtins.fromTOML rust-toolchain);
        libPath =
          with pkgs;
          lib.makeLibraryPath [
            # load external libraries that you need in your rust project here
          ];
      in
      {
        devShells.default = pkgs.mkShell rec {
          nativeBuildInputs = [ pkgs.pkg-config ];
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
          buildInputs = with pkgs; [
            openssl
            alsa-lib
          ];

          RUSTC_VERSION = overrides.toolchain.channel;

          env = {
            RUST_BACKTRACE = "full";
          };
          # https://github.com/rust-lang/rust-bindgen#environment-variables
          LIBCLANG_PATH = pkgs.lib.makeLibraryPath [ pkgs.llvmPackages_latest.libclang.lib ];

          shellHook =
            let
              proot = builtins.toString ./.;
              initFile = pkgs.writeText ".bashrc" ''
                echo "Rust shell activated!"
                set -a
                  hw() { echo "Hello world!"; }
                set +a
                # nvim .
              '';
            in
            ''
              export PATH=$PATH:''${CARGO_HOME:-~/.cargo}/bin
              export PATH=$PATH:''${RUSTUP_HOME:-~/.rustup}/toolchains/$RUSTC_VERSION-x86_64-unknown-linux-gnu/bin/
              bash --init-file ${initFile}; exit
            '';

          # Add precompiled library to rustc search path
          RUSTFLAGS = (
            builtins.map (a: ''-L ${a}/lib'') [
              # add libraries here (e.g. pkgs.libvmi)
            ]
          );

          LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath (buildInputs ++ nativeBuildInputs);

          # Add glibc, clang, glib, and other headers to bindgen search path
          BINDGEN_EXTRA_CLANG_ARGS =
            # Includes normal include path
            (builtins.map (a: ''-I"${a}/include"'') [
              # add dev libraries here (e.g. pkgs.libvmi.dev)
              pkgs.glibc.dev
            ])
            # Includes with special directory paths
            ++ [
              ''-I"${pkgs.llvmPackages_latest.libclang.lib}/lib/clang/${pkgs.llvmPackages_latest.libclang.version}/include"''
              ''-I"${pkgs.glib.dev}/include/glib-2.0"''
              ''-I${pkgs.glib.out}/lib/glib-2.0/include/''
            ];
        };
      }
    );
}
