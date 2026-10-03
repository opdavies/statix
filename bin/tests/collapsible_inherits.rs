mod _utils;

use indoc::indoc;

use macros::generate_tests;

generate_tests! {
    rule: collapsible_inherits,
    expressions: [
        // two statements of the one source
        indoc! {"
            {
              inherit (pkgs) foo;
              inherit (pkgs) bar;
            }
        "},

        // three of them, merged into the one statement
        indoc! {"
            {
              inherit (pkgs) foo;
              inherit (pkgs) bar;
              inherit (pkgs) baz;
            }
        "},

        // statements of no source merge all the same
        indoc! {"
            {
              inherit foo;
              inherit bar;
            }
        "},

        // an empty statement joins a following one
        indoc! {"
            {
              inherit;
              inherit bar;
            }
        "},

        // the run collapses to the one statement, names trailing the source
        indoc! {"
            {
              inherit (pkgs) foo bar;
              inherit (pkgs) baz;
            }
        "},

        // differing sources are two different statements
        indoc! {"
            {
              inherit (pkgs) foo;
              inherit lib;
            }
        "},

        // a blank line says the author meant to keep them apart
        indoc! {"
            {
              inherit (pkgs) foo;

              inherit (pkgs) bar;
            }
        "},

        // and so does a comment
        indoc! {"
            {
              inherit (pkgs) foo;
              # why the exception
              inherit (pkgs) bar;
            }
        "},

        // a comment inside the second statement, too
        indoc! {"
            {
              inherit (pkgs) foo;
              inherit (pkgs)
                # why the exception
                bar;
            }
        "},

        // an intervening assignment splits the run
        indoc! {"
            {
              inherit (pkgs) foo;
              name = \"x\";
              inherit (pkgs) bar;
            }
        "},

        // a statement on its own has nothing to merge into
        indoc! {"
            {
              inherit (pkgs) foo;
            }
        "},

        // inherited from inside a let, written across from an assignment
        indoc! {"
            let
              inherit (pkgs) foo;
              name = \"x\";
            in
              name
        "},
    ],
}
