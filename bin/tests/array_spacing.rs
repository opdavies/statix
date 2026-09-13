mod _utils;

use indoc::indoc;

use macros::generate_tests;

generate_tests! {
    rule: array_spacing,
    expressions: [
        // room on both sides
        indoc! {"
            {
              foo = 1;

              kernelModules = [
                \"a\"
                \"b\"
              ];

              bar = 2;
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
              bar = 2;
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

              bar = 2;
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
              bar = 2;
            }
        "},

        // first in the set, so only what follows matters
        indoc! {"
            {
              kernelModules = [
                \"a\"
                \"b\"
              ];

              foo = 1;
            }
        "},

        // alone in the set
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
              bar = 2;
            }
        "},

        // an empty array on one line
        indoc! {"
            {
              foo = 1;
              extraModulePackages = [ ];
              bar = 2;
            }
        "},
    ],
}
