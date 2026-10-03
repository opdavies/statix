{
  perSystem =
    { pkgs, ... }:
    {
      checks."statix-vim" = pkgs.statix-vim;
      treefmt.settings.global.excludes = [ "*.vim" ];
    };
}
