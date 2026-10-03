mod _utils;

use indoc::indoc;

use macros::generate_tests;

generate_tests! {
    rule: inherit_blank_line,
    expressions: [
        // an inherit crowded against what follows it
        indoc! {"
            {
              inherit system;
              owner = \"me\";
            }
        "},

        // the same, in a let binding
        indoc! {"
            let
              inherit system;
              owner = \"me\";
            in
              owner
        "},

        // a blank line is exactly what is wanted
        indoc! {"
            {
              inherit system;

              owner = \"me\";
            }
        "},

        // last in the set, with nothing after to be separated from
        indoc! {"
            {
              owner = \"me\";
              inherit system;
            }
        "},

        // written inline, so there is nowhere to put a blank line
        indoc! {"
            { inherit system; owner = \"me\"; }
        "},

        // a run of inherits is one group, checked after its last member
        indoc! {"
            {
              inherit system;
              inherit arch;
              owner = \"me\";
            }
        "},

        // the blank line may sit above a comment
        indoc! {"
            {
              inherit system;

              # where the code lives
              owner = \"me\";
            }
        "},

        // but the comment sitting against the inherit is no blank line
        indoc! {"
            {
              inherit system;
              # where the code lives
              owner = \"me\";
            }
        "},

        // two groups, each wanting a blank line after its last member
        indoc! {"
            {
              inherit system;
              owner = \"me\";
              inherit name;
              paths = [ ];
            }
        "},

        // an inherit buried in a nested block is found too
        indoc! {"
            {
              inputs = {
                inherit system;
                owner = \"me\";
              };
            }
        "},
    ],
}
