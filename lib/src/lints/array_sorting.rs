use crate::{Metadata, Report, Rule, Suggestion, utils};

use macros::lint;
use rnix::{NodeOrToken, Root, SyntaxElement, SyntaxKind, SyntaxNode, ast::List};
use rowan::ast::AstNode as _;

/// ## What it does
/// Checks that the elements of a multiline array are in alphabetical order.
///
/// ## Why is this bad?
/// Kernel modules, mount options, package lists and imports are all sets of
/// things rather than sequences, so their order carries no meaning and may
/// as well be one that makes an entry easy to find.
///
/// Ordering is by byte, matching `:sort` in vim and `LC_ALL=C sort`, as it
/// is for attributes.
///
/// Only arrays whose elements are each written on one line are considered.
/// Where an element spans lines it is some larger expression, and the order
/// of those is much more likely to matter.
///
/// ## Example
/// ```nix
/// [
///   "nvme"
///   "ahci"
/// ]
/// ```
///
/// Sort them.
/// ```nix
/// [
///   "ahci"
///   "nvme"
/// ]
/// ```
#[lint(
    name = "array_sorting",
    note = "Array elements are not in alphabetical order",
    code = 34,
    match_with = SyntaxKind::NODE_LIST
)]
struct ArraySorting;

impl Rule for ArraySorting {
    fn validate(&self, node: &SyntaxElement) -> Option<Report> {
        let NodeOrToken::Node(node) = node else {
            return None;
        };

        if !utils::is_multiline(node) {
            return None;
        }

        let parts = utils::segments(node);

        // A single element is in order by definition, and one spanning lines
        // means this is not the kind of list worth sorting.
        if parts.items.len() < 2
            || parts
                .items
                .iter()
                .any(|element| utils::is_multiline(&element.node))
        {
            return None;
        }

        let text_of = |element: &SyntaxNode| element.text().to_string();

        let out_of_place = parts
            .items
            .windows(2)
            .find(|pair| text_of(&pair[0].node) > text_of(&pair[1].node))
            .map(|pair| pair[1].node.clone())?;

        let mut order: Vec<usize> = (0..parts.items.len()).collect();
        order.sort_by_key(|&position| text_of(&parts.items[position].node));

        let text = utils::rebuild(&parts, &order);

        let sorted = Root::parse(&text)
            .syntax()
            .descendants()
            .find_map(|node| List::cast(node).map(|list| list.syntax().clone()))?;

        Some(self.report().suggest(
            out_of_place.text_range(),
            format!("`{}` is out of alphabetical order", text_of(&out_of_place)),
            Suggestion::with_replacement(node.text_range(), sorted),
        ))
    }
}
