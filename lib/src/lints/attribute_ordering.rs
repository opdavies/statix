use crate::{Metadata, Report, Rule, utils};

use macros::lint;
use rnix::{
    NodeOrToken, SyntaxElement, SyntaxKind,
    ast::{AttrSet, HasEntry as _},
};
use rowan::ast::AstNode as _;

/// ## What it does
/// Checks that the attributes of a set are in alphabetical order.
///
/// ## Why is this bad?
/// A set that is ordered by nothing in particular gives a reader no way to
/// find an attribute except to read all of them, and gives two people
/// editing it no agreed place to add one.
///
/// Ordering is by byte, which is what `:sort` in vim and `LC_ALL=C sort`
/// both produce. Uppercase therefore sorts before lowercase, so `DOMAIN`
/// comes before `apple`. The locale-aware ordering of a bare `sort` is
/// deliberately not used, since it would have the lint disagree with itself
/// across machines.
///
/// `enable` and its camelCase relatives are exempt. They belong at the top
/// of the set as a group, which the `enable_first` lint covers.
///
/// ## Example
/// ```nix
/// {
///   zebra = true;
///   apple = true;
/// }
/// ```
///
/// Sort them.
/// ```nix
/// {
///   apple = true;
///   zebra = true;
/// }
/// ```
#[lint(
    name = "attribute_ordering",
    note = "Attributes are not in alphabetical order",
    code = 31,
    match_with = SyntaxKind::NODE_ATTR_SET
)]
struct AttributeOrdering;

impl Rule for AttributeOrdering {
    fn validate(&self, node: &SyntaxElement) -> Option<Report> {
        let NodeOrToken::Node(node) = node else {
            return None;
        };

        let attr_set = AttrSet::cast(node.clone())?;

        let ordered: Vec<_> = attr_set
            .entries()
            .filter_map(utils::as_assignment)
            .filter_map(|assignment| {
                let name = utils::attribute_name(&assignment)?;

                Some((name, assignment))
            })
            .filter(|(name, _)| !utils::is_enable_family(name))
            .collect();

        let (name, out_of_place) = ordered
            .windows(2)
            .find(|pair| pair[0].0 > pair[1].0)
            .map(|pair| (&pair[1].0, &pair[1].1))?;

        Some(self.report().diagnostic(
            out_of_place.syntax().text_range(),
            format!("`{name}` is out of alphabetical order"),
        ))
    }
}
