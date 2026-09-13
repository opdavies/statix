mod _utils;

use indoc::indoc;

use macros::generate_tests;

generate_tests! {
    rule: multiline_spacing,
    expressions: [
        // room on both sides
        indoc! {"
            {
              foo = 1;

              kernelModules = [
                \"a\"
                \"b\"
              ];

              zed = 2;
            }
        "},

        // crowded on both sides
        indoc! {"
            {
              foo = 1;
              kernelModules = [
                \"a\"
                \"b\"
              ];
              zed = 2;
            }
        "},

        // crowded before only
        indoc! {"
            {
              foo = 1;
              kernelModules = [
                \"a\"
                \"b\"
              ];

              zed = 2;
            }
        "},

        // crowded after only
        indoc! {"
            {
              foo = 1;

              kernelModules = [
                \"a\"
                \"b\"
              ];
              zed = 2;
            }
        "},

        // first in the set, so only what follows matters
        indoc! {"
            {
              kernelModules = [
                \"a\"
                \"b\"
              ];

              zed = 1;
            }
        "},

        // last in the set, so only what precedes matters
        indoc! {"
            {
              foo = 1;

              kernelModules = [
                \"a\"
                \"b\"
              ];
            }
        "},

        // alone in the set, with nothing on either side
        indoc! {"
            {
              kernelModules = [
                \"a\"
                \"b\"
              ];
            }
        "},

        // written on one line, so it is an ordinary setting
        indoc! {"
            {
              foo = 1;
              kernelModules = [ \"a\" ];
              zed = 2;
            }
        "},

        // a nested block is a multiline attribute too
        indoc! {"
            {
              foo = 1;
              publish = {
                x = 1;
              };
              zed = 2;
            }
        "},

        // and so is a multiline string
        indoc! {"
            {
              name = \"x\";
              text = ''
                hello
              '';
              zed = 1;
            }
        "},

        // and a function application written across lines
        indoc! {"
            {
              foo = 1;
              package = pkgs.writeShellApplication {
                name = \"x\";
              };
              zed = 2;
            }
        "},

        // Two neighbours describe the one gap between them, once from each
        // side. It must end up a single blank line rather than two.
        indoc! {"
            {
              alpha = [
                \"a\"
                \"b\"
              ];
              beta = [
                \"c\"
                \"d\"
              ];
            }
        "},

        // a value merely wrapped onto the next line is not multiline: the
        // break sits before the value rather than inside it
        indoc! {"
            {
              a = 1;
              foo =
                someLongThing;
              zed = 2;
            }
        "},
    ],
}
