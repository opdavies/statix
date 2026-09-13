mod _utils;

use indoc::indoc;

use macros::generate_tests;

generate_tests! {
    rule: enable_first,
    expressions: [
        // already first
        indoc! {"
            {
              enable = true;
              nssmdns4 = true;
            }
        "},

        // the only attribute, so nothing to come before it
        "{ enable = true; }",

        // no enable at all
        indoc! {"
            {
              nssmdns4 = true;
              openFirewall = true;
            }
        "},

        // buried among the settings it governs
        indoc! {"
            {
              nssmdns4 = true;
              enable = true;
              openFirewall = true;
            }
        "},

        // last
        indoc! {"
            {
              nssmdns4 = true;
              openFirewall = true;
              enable = true;
            }
        "},

        // an inherit takes no part in the ordering
        indoc! {"
            {
              inherit lib;
              enable = true;
              nssmdns4 = true;
            }
        "},

        // `enableCompletion` is its own attribute, not this one
        indoc! {"
            {
              enableCompletion = true;
              enable = true;
            }
        "},

        // a dotted path belongs to the set it names
        indoc! {"
            {
              syntaxHighlighting.enable = true;
              enable = true;
            }
        "},

        // nested sets are judged separately
        indoc! {"
            {
              enable = true;
              initrd = {
                kernelModules = [ ];
                enable = true;
              };
            }
        "},
    ],
}
