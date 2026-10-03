{
  perSystem =
    { pkgs, ... }:
    {
      checks.build = pkgs.statix;
      treefmt.settings.global.excludes = [ "bin/tests/data/*.nix" ];
    };
}
