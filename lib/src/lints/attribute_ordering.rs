use crate::{Metadata, Report, Rule, Suggestion, utils};

use macros::lint;
use rnix::{
    NodeOrToken, Root, SyntaxElement, SyntaxKind, SyntaxNode,
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
            .map(|pair| (pair[1].0.clone(), pair[1].1.syntax().clone()))?;

        let at = out_of_place.text_range();
        let message = format!("`{name}` is out of alphabetical order");

        // A set this cannot rewrite is still reported, just not fixed.
        let Some(sorted) = sort_entries(node) else {
            return Some(self.report().diagnostic(at, message));
        };

        Some(self.report().suggest(
            at,
            message,
            Suggestion::with_replacement(node.text_range(), sorted),
        ))
    }
}

/// Rewrite the set with its attributes in order.
///
/// Sorting only the assignments would move them past an `inherit`, which is
/// a different kind of entry and stays where the author put it. Only the
/// positions the assignments already occupy are reordered among themselves.
///
/// Each attribute keeps the trivia that preceded it, so a comment above one
/// moves with it. The trivia is held in place while the attributes move
/// beneath it, which keeps the indentation and any blank line belonging to a
/// position rather than dragging it around.
fn sort_entries(node: &SyntaxNode) -> Option<SyntaxNode> {
    let parts = utils::segments(node);
    let sortable = utils::sortable_positions(&parts.items);

    let mut sorted = sortable.clone();
    sorted.sort_by_key(|&position| sort_key(&parts.items[position].node));

    // Each sortable entry takes the next sortable position. An `inherit`,
    // and everything the ordering skips, stays exactly where it was.
    let mut order: Vec<usize> = (0..parts.items.len()).collect();

    for (&target, &source) in sortable.iter().zip(&sorted) {
        order[target] = source;
    }

    let text = utils::rebuild(&parts, &order);

    Root::parse(&text)
        .syntax()
        .descendants()
        .find_map(|candidate| AttrSet::cast(candidate).map(|set| set.syntax().clone()))
}

fn sort_key(entry: &SyntaxNode) -> Option<String> {
    rnix::ast::AttrpathValue::cast(entry.clone())
        .and_then(|assignment| utils::attribute_name(&assignment))
}
