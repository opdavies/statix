use crate::{Metadata, Report, Rule, Suggestion, utils};

use macros::lint;
use rnix::{
    NodeOrToken, Root, SyntaxElement, SyntaxKind, SyntaxNode,
    ast::{AttrSet, AttrpathValue, HasEntry as _},
};
use rowan::ast::AstNode as _;

/// ## What it does
/// Checks that `enable` is the first attribute in an attribute set.
///
/// ## Why is this bad?
/// Whether a block is switched on at all is the first thing a reader needs
/// to know, so it reads better at the top than buried among the settings it
/// governs.
///
/// ## Example
/// ```nix
/// {
///   nssmdns4 = true;
///   openFirewall = true;
///   enable = true;
/// }
/// ```
///
/// Put `enable` first.
/// ```nix
/// {
///   enable = true;
///
///   nssmdns4 = true;
///   openFirewall = true;
/// }
/// ```
#[lint(
    name = "enable_first",
    note = "`enable` is not the first attribute",
    code = 30,
    match_with = SyntaxKind::NODE_ATTR_SET
)]
struct EnableFirst;

impl Rule for EnableFirst {
    fn validate(&self, node: &SyntaxElement) -> Option<Report> {
        let NodeOrToken::Node(node) = node else {
            return None;
        };

        let attr_set = AttrSet::cast(node.clone())?;
        let mut assignments = attr_set.entries().filter_map(utils::as_assignment);

        // An `inherit` takes no part in this. Only the assignments are
        // ordered, so the first of those is the position `enable` wants.
        let first = assignments.next()?;

        if utils::is_enable(&first) {
            return None;
        }

        let enable = assignments.find(utils::is_enable)?;

        let at = enable.syntax().text_range();
        let message = "`enable` should be the first attribute in this set";

        // The report does not depend on the rewrite succeeding. Were it to,
        // a set this could not rewrite would go unreported rather than
        // merely unfixed.
        let Some(moved) = move_enable_first(node) else {
            return Some(self.report().diagnostic(at, message));
        };

        Some(self.report().suggest(
            at,
            message,
            Suggestion::with_replacement(node.text_range(), moved),
        ))
    }
}

/// Rewrite the set with `enable` at the front of the attributes.
///
/// Only the positions the attributes already occupy are used, so an
/// `inherit` keeps its place. Everything else keeps its relative order,
/// which leaves any existing sorting alone for `attribute_ordering` to
/// judge separately.
fn move_enable_first(node: &SyntaxNode) -> Option<SyntaxNode> {
    let parts = utils::segments(node);
    let positions = utils::assignment_positions(&parts.items);

    let enable_at = positions.iter().position(|&position| {
        AttrpathValue::cast(parts.items[position].node.clone())
            .is_some_and(|assignment| utils::is_enable(&assignment))
    })?;

    let mut sources = positions.clone();
    let enable = sources.remove(enable_at);
    sources.insert(0, enable);

    let mut order: Vec<usize> = (0..parts.items.len()).collect();

    for (&target, &source) in positions.iter().zip(&sources) {
        order[target] = source;
    }

    let text = utils::rebuild(&parts, &order);

    Root::parse(&text)
        .syntax()
        .descendants()
        .find_map(|candidate| AttrSet::cast(candidate).map(|set| set.syntax().clone()))
}
