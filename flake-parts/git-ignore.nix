{ lib, config, ... }:
{
  config = {
    gitignore = [ "/result" ];

    perSystem = {
      files.file.".gitignore".text = config.gitignore;
    };
  };

  options.gitignore = lib.mkOption {
    apply = x: lib.concatLines (lib.naturalSort x);

    type = lib.types.listOf lib.types.singleLineStr;
  };
}
