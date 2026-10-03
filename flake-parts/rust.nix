{
  gitignore = [ "/target" ];

  perSystem =
    { pkgs, ... }:
    {
      make-shells.default = {
        env = {
          RUST_BACKTRACE = 1;
          RUST_LOG = "info";
        };

        inputsFrom = [ pkgs.statix ];

        packages = [
          pkgs.bacon
          pkgs.cargo-insta
          pkgs.rust-analyzer
        ];
      };

      treefmt = {
        programs.rustfmt.enable = true;

        settings.global.excludes = [
          "bin/tests/snapshots/*.snap"
        ];
      };
    };
}
