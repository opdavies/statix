{
  vimUtils,
  lib,
}:
let
  pluginRoot = ../vim-plugin;
in
vimUtils.buildVimPlugin {
  pname = "statix-vim";

  src = lib.fileset.toSource {
    fileset = lib.fileset.union (pluginRoot + "/plugin/statix.vim") (pluginRoot + "/ftplugin/nix.vim");
    root = pluginRoot;
  };

  version = "0.1.0-git";
}
