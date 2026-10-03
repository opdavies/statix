mod _utils;

use indoc::indoc;

use macros::generate_tests;

generate_tests! {
    rule: blank_lines,
    expressions: [
        // a single blank line is exactly right
        indoc! {"
            {
              alpha = 1;

              beta = 2;
            }
        "},

        // two blank lines
        indoc! {"
            {
              alpha = 1;


              beta = 2;
            }
        "},

        // three blank lines
        indoc! {"
            {
              alpha = 1;



              beta = 2;
            }
        "},

        // blank lines in a nested block
        indoc! {"
            {
              outer = {
                alpha = 1;


                beta = 2;
              };
            }
        "},

        // a single blank line before the closing brace
        indoc! {"
            {
              alpha = 1;

              beta = 2;

            }
        "},

        // two blank lines before the closing brace
        indoc! {"
            {
              alpha = 1;

              beta = 2;


            }
        "},

        // a blank line before the closing bracket of a list
        indoc! {"
            {
              letters = [
                \"a\"
                \"b\"

              ];
            }
        "},

        // blank lines inside a multiline string are not layout and must stay
        indoc! {"
            {
              text = ''
                foo


                bar
              '';
            }
        "},

        // the same, on a string that is otherwise well spaced
        indoc! {"
            {
              alpha = 1;

              text = ''
                foo


                bar
              '';
            }
        "},

        // blank lines via indented strings in an expression, not a string
        indoc! {"
            let
              alpha = 1;


              beta = 2;
            in
              foo + bar
        "},
    ],
}
