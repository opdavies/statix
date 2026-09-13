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
    ],
}
