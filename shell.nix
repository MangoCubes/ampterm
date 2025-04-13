let
  moz_overlay = import (
    builtins.fetchTarball "https://github.com/mozilla/nixpkgs-mozilla/archive/master.tar.gz"
  );
  pkgs = import <nixpkgs> { overlays = [ moz_overlay ]; };
in
pkgs.mkShell {
  packages = (
    with pkgs;
    [
      rust-analyzer
      lldb
      rustup
      jq
    ]
  );
  buildInputs = (
    with pkgs;
    [
      openssl
      alsa-lib
      latest.rustChannels.nightly.rust
      pkg-config
    ]
  );
  # Editor is nvim
  env = {
    RUST_BACKTRACE = "full";
  };
  RUST_SRC_PATH = "${pkgs.rust.packages.stable.rustPlatform.rustLibSrc}";
  LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath [ pkgs.openssl ];
}
