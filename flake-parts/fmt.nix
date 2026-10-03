{ inputs, ... }:
{
  imports = [ "${inputs.treefmt}/flake-module.nix" ];

  perSystem =
    psArgs@{ pkgs, ... }:
    {
      pre-commit.settings.hooks.treefmt.enable = true;

      treefmt = {
        programs = {
          nixfmt = {
            enable = true;

            package = pkgs.nixfmt;
          };

          prettier.enable = true;

          taplo = {
            enable = true;

            settings.formatting = {
              allowed_blank_lines = 1;
              reorder_arrays = true;
              reorder_inline_tables = true;
              reorder_keys = true;
            };
          };
        };

        projectRootFile = "flake.nix";
        settings.on-unmatched = "fatal";
      };
    };
}
