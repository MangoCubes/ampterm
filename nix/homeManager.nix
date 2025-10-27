{
  config,
  self,
  pkgs,
  ...
}:
let
  cfg = config.niri-adv-rules;
in
{
  home.packages = [
    self.packages."${pkgs.stdenv.hostPlatform.system}".default
  ];
}
