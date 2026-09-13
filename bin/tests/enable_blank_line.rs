mod _utils;

use indoc::indoc;

use macros::generate_tests;

generate_tests! {
    rule: enable_blank_line,
    expressions: [
        // separated
        indoc! {"
            {
              enable = true;

              nssmdns4 = true;
            }
        "},

        // not separated
        indoc! {"
            {
              enable = true;
              nssmdns4 = true;
            }
        "},

        // the only attribute, so nothing to separate from
        indoc! {"
            {
              enable = true;
            }
        "},

        // the family is one group, with the blank line after the last of it
        indoc! {"
            {
              enable = true;
              enableCompletion = true;

              cdpath = \"a\";
            }
        "},

        // no blank line after the group
        indoc! {"
            {
              enable = true;
              enableCompletion = true;
              cdpath = \"a\";
            }
        "},

        // written inline, so there is nowhere to put a blank line
        "{ enable = true; nssmdns4 = true; }",

        // no enable at all
        indoc! {"
            {
              nssmdns4 = true;
              openFirewall = true;
            }
        "},

        // `enable` further down is `enable_first`'s business, not this one's
        indoc! {"
            {
              nssmdns4 = true;
              enable = true;
            }
        "},

        // a blank line above the comment still separates the two
        indoc! {"
            {
              enable = true;

              # about nssmdns4
              nssmdns4 = true;
            }
        "},

        // a comment with no blank line anywhere does not separate them
        indoc! {"
            {
              enable = true;
              # about nssmdns4
              nssmdns4 = true;
            }
        "},
    ],
}
