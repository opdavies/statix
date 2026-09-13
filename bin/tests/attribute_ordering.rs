mod _utils;

use indoc::indoc;

use macros::generate_tests;

generate_tests! {
    rule: attribute_ordering,
    expressions: [
        // sorted
        indoc! {"
            {
              apple = true;
              zebra = true;
            }
        "},

        // out of order
        indoc! {"
            {
              zebra = true;
              apple = true;
            }
        "},

        // a single attribute has nothing to be out of order with
        "{ zebra = true; }",

        // byte order, as `:sort` and `LC_ALL=C sort` give it: uppercase
        // first, then underscore, then lowercase
        indoc! {"
            {
              Banana = true;
              DOMAIN = true;
              _module = true;
              apple = true;
            }
        "},

        // the same names in locale order, which is not what is wanted
        indoc! {"
            {
              apple = true;
              Banana = true;
              DOMAIN = true;
              _module = true;
            }
        "},

        // the enable family is exempt and sorts nowhere
        indoc! {"
            {
              enable = true;
              enableCompletion = true;

              apple = true;
              zebra = true;
            }
        "},

        // `enabledCollectors` is the word \"enabled\", so it sorts normally
        indoc! {"
            {
              zebra = true;
              enabledCollectors = [ ];
            }
        "},

        // dotted paths sort on the whole path
        indoc! {"
            {
              services.zebra = true;
              services.apple = true;
            }
        "},

        // an inherit takes no part in the ordering
        indoc! {"
            {
              inherit lib;
              apple = true;
              zebra = true;
            }
        "},

        // one report per set, not one per pair
        indoc! {"
            {
              zebra = true;
              yak = true;
              apple = true;
            }
        "},

        // nested sets are judged separately
        indoc! {"
            {
              apple = {
                zebra = true;
                ant = true;
              };
              zebra = true;
            }
        "},

        // a comment sits above the attribute it describes
        indoc! {"
            {
              # about zebra
              zebra = true;
              # about apple
              apple = true;
            }
        "},
    ],
}
