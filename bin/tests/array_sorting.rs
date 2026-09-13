mod _utils;

use indoc::indoc;

use macros::generate_tests;

generate_tests! {
    rule: array_sorting,
    expressions: [
        // sorted
        indoc! {"
            [
              \"ahci\"
              \"nvme\"
            ]
        "},

        // out of order
        indoc! {"
            [
              \"nvme\"
              \"ahci\"
            ]
        "},

        // one element is in order by definition
        indoc! {"
            [
              \"ahci\"
            ]
        "},

        // written on one line, where the order is the author's business
        "[ \"nvme\" \"ahci\" ]",

        // byte order, so uppercase comes first
        indoc! {"
            [
              \"Zebra\"
              \"apple\"
            ]
        "},

        // the same two in locale order, which is not what is wanted
        indoc! {"
            [
              \"apple\"
              \"Zebra\"
            ]
        "},

        // a package list
        indoc! {"
            [
              pkgs.git
              pkgs.vim
            ]
        "},

        // an unsorted package list
        indoc! {"
            [
              pkgs.vim
              pkgs.git
            ]
        "},

        // an element spanning lines means this is not a list of names
        indoc! {"
            [
              (pkgs.writeShellApplication {
                name = \"zebra\";
              })
              pkgs.apple
            ]
        "},

        // a comment travels with the element it sits above
        indoc! {"
            [
              # about nvme
              \"nvme\"
              \"ahci\"
            ]
        "},

        // a comment on each of them
        indoc! {"
            [
              # about nvme
              \"nvme\"
              # about ahci
              \"ahci\"
            ]
        "},

        // a string is compared by its content, so the one the formatter had
        // to write as ''...'' sorts among the rest rather than apart from
        // them
        indoc! {"
            [
              ''if [[ \"$1\" == \"\" ]]; then''
              \"message=$2\"
              \"nohup _timer $mins\"
            ]
        "},

        // the same three, out of order by content
        indoc! {"
            [
              \"nohup _timer $mins\"
              ''if [[ \"$1\" == \"\" ]]; then''
              \"message=$2\"
            ]
        "},

        // two arrays read against one another by position, where sorting
        // one alone would pair each string with the wrong replacement
        indoc! {"
            builtins.replaceStrings
              [
                \"zebra\"
                \"apple\"
              ]
              [
                \"stripes\"
                \"core\"
              ]
        "},

        // an attribute named order is a sequence
        indoc! {"
            {
              order = [
                \"intro\"
                \"hosts\"
              ];
            }
        "},

        // the last attribute of a dotted path is the one naming the array
        indoc! {"
            {
              text.readme.order = [
                \"intro\"
                \"hosts\"
              ];
            }
        "},

        // any other attribute is sorted as usual
        indoc! {"
            {
              modules = [
                \"zebra\"
                \"apple\"
              ];
            }
        "},

        // a string keeps its place against an expression beside it, rather
        // than being compared to one by its content alone
        indoc! {"
            [
              \"PATH\"
              (lib.makeBinPath [ hello ])
            ]
        "},

        // and against an identifier
        indoc! {"
            [
              \"zebra\"
              pkgs.apple
            ]
        "},
    ],
}
