mod _utils;

use indoc::indoc;

use macros::generate_tests;

generate_tests! {
    rule: inherit_ordering,
    expressions: [
        // the user's example, with `mkOption` and `mkPackageOption` swapped
        indoc! {"
            {
              inherit (lib)
                mkEnableOption
                mkIf
                mkMerge
                mkPackageOption
                mkOption
                ;
            }
        "},

        // already sorted
        indoc! {"
            {
              inherit (lib)
                mkEnableOption
                mkIf
                mkMerge
                mkOption
                mkPackageOption
                ;
            }
        "},

        // a single-line `inherit` is judged as well as a multiline one
        "{ inherit b a; }",

        // the from-expression is not a name and stays where it is
        "{ inherit (lib) mkMerge mkIf; }",

        // a single name is in order by definition
        "{ inherit (inputs) self; }",

        // no from-expression at all
        "{ inherit lib pkgs inputs; }",

        // byte order, as `:sort` and `LC_ALL=C sort` give it: uppercase
        // first, then underscore, then lowercase
        indoc! {"
            {
              inherit
                apple
                DOMAIN
                Banana
                _module
                ;
            }
        "},

        // several inherits are sorted independently of one another
        indoc! {"
            {
              inherit (lib) mkMerge mkIf;
              inherit (inputs) self nixpkgs;
              inherit zebra apple;
            }
        "},

        // names quoted or drawn with `${…}` are sorted by the text they
        // are written with, like the plain identifiers beside them
        "{ inherit zebra yak; inherit yak ${apple}; }",

        // a comment sits above the name it describes
        indoc! {"
            {
              inherit (lib)
                # about mkMerge
                mkMerge
                # about mkIf
                mkIf
                ;
            }
        "},

        // blank lines between the entries are the affair of
        // `inherit_blank_line` and the statements' order of
        // `inherit_first`; however they are laid out, each statement is
        // sorted within itself
        indoc! {"
            {
              inherit apple zebra
                middle;
              other = true;
            }
        "},
    ],
}
