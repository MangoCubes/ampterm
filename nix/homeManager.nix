{
  config,
  inputs,
  pkgs,
  lib,
  ...
}:
{
  options = {
    programs.ampterm = {
      enable = lib.mkOption {
        type = lib.types.bool;
        default = false;
        description = ''Enable Ampterm, a TUI-based OpenSubsonic client.'';
      };
      settings = lib.mkOption {
        type = lib.types.attrs;
        default = { };
        description = ''Ampterm config'';
      };
    };
  };
  config =
    let
      cfg = config.programs.ampterm;
    in
    lib.mkIf cfg.enable {
      home.packages = [
        inputs.ampterm.packages."${pkgs.stdenv.hostPlatform.system}".default
      ];
      xdg.configFile."ampterm/config.json".text = (builtins.toJSON cfg.settings);
    };
}
