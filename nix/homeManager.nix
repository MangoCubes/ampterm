{
  config,
  inputs,
  pkgs,
  lib,
  ...
}:
let
  cfg = config.ampterm;
in
{
  options = {
    programs.ampterm = {
      enable = lib.mkOption {
        type = lib.types.bool;
        default = false;
        description = ''Enable Ampterm, a TUI-based OpenSubsonic client.'';
      };
    };
  };
  config = lib.mkIf cfg.enable {
    home.packages = [
      inputs.ampterm.packages."${pkgs.stdenv.hostPlatform.system}".default
    ];
  };
}
