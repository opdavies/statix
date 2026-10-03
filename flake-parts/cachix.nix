{ lib, ... }:
{
  perSystem =
    { pkgs, ... }:
    {
      apps.cachix-push = {
        program = lib.getExe (
          pkgs.writeShellApplication {
            name = "cachix-push";

            runtimeInputs = with pkgs; [
              cachix
              jq
              nix
            ];

            text = ''
              nix build --json \
              | jq -r '.[].outputs | to_entries[].value' \
              | cachix push statix
            '';
          }
        );

        type = "app";
      };
    };
}
