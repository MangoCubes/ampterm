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
        type = lib.types.submodule {
          options.keybindings = lib.mkOption {
            type = lib.types.submodule {
              options.Visual = lib.mkOption {
                type = lib.types.attrs;
                default = { };
              };
              options.Normal = lib.mkOption {
                type = lib.types.attrs;
                default = { };
              };
              options.Common = lib.mkOption {
                type = lib.types.attrs;
                default = { };
              };
            };
          };
          options.use_legacy_auth = lib.mkOption {
            type = lib.types.bool;
            default = false;
            description = ''If set true, then the player will log in over HTTP.'';
          };
          options.unsafe_auth = lib.mkOption {
            description = ''Login credentials in plaintext.'';
            type = lib.types.submodule {
              options.url = lib.mkOption {
                type = lib.types.str;
                default = "";
                description = ''URL of the OpenSubsonic server in plaintext.'';
              };
              options.username = lib.mkOption {
                type = lib.types.str;
                default = "";
                description = ''Username of your OpenSubsonic server credentials in plaintext.'';
              };
              options.password = lib.mkOption {
                type = lib.types.str;
                default = "";
                description = ''Password of your OpenSubsonic server credentials in plaintext.'';
              };
            };
            default = {
              url = "";
              username = "";
              password = "";
            };
          };
          options.auth = lib.mkOption {
            description = ''Set of commands that will be run to get your OpenSubsonic server. If you wish to store some of them in plaintext, use "echo <text>".'';
            type = lib.types.submodule {
              options.url = {
                type = lib.types.str;
                default = "";
              };
              options.username = {
                type = lib.types.str;
                default = "";
              };
              options.password = {
                type = lib.types.str;
                default = "";
              };
            };
            default = {
              url = "";
              username = "";
              password = "";
            };
          };
        };
        default = { };
        description = ''Ampterm config goes here.'';
      };
      extraOptions = lib.mkOption {
        type = lib.types.attrs;
        default = { };
        description = ''Any options that are not yet implemented in the flake goes here.'';
      };
    };
  };
  config =
    let
      cfg = config.programs.ampterm;
    in
    lib.mkIf cfg.enable {
      # Install package
      home.packages = [
        inputs.ampterm.packages."${pkgs.stdenv.hostPlatform.system}".default
      ];
      xdg = {
        # Create config.json
        configFile."ampterm/config.json".text = (builtins.toJSON (cfg.settings // cfg.extraOptions));
        # Create XDG entry
        desktopEntries.ampterm = {
          name = "Ampterm";
          genericName = "Terminal Music Player";
          exec = ''ampterm'';
          terminal = true;
        };
      };
    };
}
