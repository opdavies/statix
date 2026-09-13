use crate::{Metadata, Report, Rule, utils};

use macros::lint;
use rnix::{NodeOrToken, SyntaxElement, SyntaxKind, ast::List};
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

        let list = List::cast(node.clone())?;

        if !utils::is_multiline(node) {
            return None;
        }

        let elements: Vec<_> = list
            .items()
            .map(|item| (item.syntax().text().to_string(), item))
            .collect();

        // A single element is in order by definition, and one spanning lines
        // means this is not the kind of list worth sorting.
        if elements.len() < 2 || elements.iter().any(|(text, _)| text.contains('\n')) {
            return None;
        }

        let (text, out_of_place) = elements
            .windows(2)
            .find(|pair| pair[0].0 > pair[1].0)
            .map(|pair| (&pair[1].0, &pair[1].1))?;

        Some(self.report().diagnostic(
            out_of_place.syntax().text_range(),
            format!("`{text}` is out of alphabetical order"),
        ))
    }
}
