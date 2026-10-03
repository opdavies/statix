{ inputs, ... }:
{
  gitignore = [
    "/.pre-commit-config.yaml"
  ];

  imports = [ "${inputs.git-hooks}/flake-module.nix" ];

  perSystem =
    { config, ... }:
    {
      make-shells.default.shellHook = config.pre-commit.installationScript;
      pre-commit.check.enable = false;
      treefmt.settings.global.excludes = [ ".pre-commit-config.yaml" ];
    };
}
