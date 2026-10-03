{
  inputs = {
    files = {
      flake = false;
      url = "github:mightyiam/files";
    };

    flake-parts = {
      inputs.nixpkgs-lib.follows = "nixpkgs";
      url = "github:hercules-ci/flake-parts";
    };

    git-hooks = {
      flake = false;
      url = "github:cachix/git-hooks.nix";
    };

    import-tree = {
      flake = false;
      url = "github:denful/import-tree";
    };

    make-shell = {
      flake = false;
      url = "github:nicknovitski/make-shell";
    };

    nixpkgs = {
      url = "https://channels.nixos.org/nixpkgs-unstable/nixexprs.tar.xz";
    };

    treefmt = {
      flake = false;
      url = "github:numtide/treefmt-nix";
    };
  };

  nixConfig = {
    abort-on-warn = true;
    allow-import-from-derivation = false;
  };

  outputs =
    inputs:
    inputs.flake-parts.lib.mkFlake { inherit inputs; } (
      { lib, ... }:
      {
        _module.args.root = ./.;

        imports = [ (import inputs.import-tree ./flake-parts) ];
      }
    );
}
