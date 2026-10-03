mod _utils;

use indoc::indoc;

use macros::generate_tests;

generate_tests! {
    rule: inherit_first,
    expressions: [
        // an inherit first in the set is already as wanted
        indoc! {"
            {
              inherit lib;
              apple = true;
            }
        "},

        // an inherit after an assignment is out of place
        indoc! {"
            {
              spellcheck = pkgs.callPackage ./spellcheck.pkg.nix { };
              inherit slugifier;
              test = pkgs.callPackage ./test.pkg.nix { };
            }
        "},

        // an inherit mid-set and one at the bottom, moved together,
        // keeping the order they were written in
        indoc! {"
            {
              apple = true;
              inherit (pkgs) zebra;
              yak = true;
              inherit lib;
            }
        "},

        // a group of inherits followed later by another: the later one
        // joins the group at the top
        indoc! {"
            {
              inherit lib;
              apple = true;
              yak = true;
              inherit (pkgs) zebra;
            }
        "},

        // an inherit in a let binding is judged the same
        indoc! {"
            let
              name = \"x\";
              inherit (pkgs) foo;
            in
              name
        "},

        // the body of a let stays where it is, which is the only
        // non-entry position the rewrite walks over
        indoc! {"
            let
              inherit lib;
              name = \"x\";
            in
              name
        "},

        // a set of only inherits has nothing to come before
        indoc! {"
            {
              inherit lib;
              inherit (pkgs) zebra;
            }
        "},

        // a set written inline has no reading order to speak of
        "{ spellcheck = \"a\"; inherit slugifier; }",

        // a comment above an inherit moves with it
        indoc! {"
            {
              apple = true;
              # about slugifier
              inherit slugifier;
              test = true;
            }
        "},

        // a comment above an assignment keeps describing it, from its
        // new place
        indoc! {"
            {
              # about test
              test = true;
              inherit slugifier;
              spellcheck = true;
            }
        "},

        // `rec` has to survive the rewrite
        indoc! {"
            stdenv.mkDerivation rec {
              version = \"1.1.1\";
              inherit (pkgs) pname;
            }
        "},

        // nesting goes in sets, not in the inherits
        indoc! {"
            {
              inherit lib;
              attrs = {
                apple = true;
                inherit zebra;
              };
            }
        "},

        // the enable family follows the inherits, which is what
        // `enable_first` wants given this one
        indoc! {"
            {
              enable = true;

              inherit slugifier;
            }
        "},

        // an inherit last in the set, but not first: still out of place
        // against the assignments above it
        indoc! {"
            {
              apple = true;
              inherit lib;
            }
        "},
    ],
}
