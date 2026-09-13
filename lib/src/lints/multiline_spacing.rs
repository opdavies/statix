use crate::{Metadata, Report, Rule, Suggestion, utils};

use macros::lint;
use rnix::{
    NodeOrToken, Root, SyntaxElement, SyntaxKind, SyntaxNode,
    ast::{AttrSet, AttrpathValue, HasEntry as _},
};
use rowan::ast::AstNode as _;

/// ## What it does
/// Checks that an attribute whose value spans more than one line is
/// separated from its neighbours by a single blank line.
///
/// ## Why is this bad?
/// Something written across several lines is a block rather than a one-line
/// setting, and reads as one when it is given room. Without the blank lines
/// its first and last lines crowd against unrelated attributes.
///
/// This covers nested blocks, multiline arrays, multiline strings and
/// anything else written across lines. An attribute first or last in its set
/// has nothing on that side to be separated from.
///
/// The blank line belongs before the attribute, not between its `=` and the
/// `[` or `{` opening its value, since nothing can sit there.
///
/// ## Example
/// ```nix
/// {
///   foo = 1;
///   kernelModules = [
///     "a"
///     "b"
///   ];
///   bar = 2;
/// }
/// ```
///
/// Give it room.
/// ```nix
/// {
///   foo = 1;
///
///   kernelModules = [
///     "a"
///     "b"
///   ];
///
///   bar = 2;
/// }
/// ```
#[lint(
    name = "multiline_spacing",
    note = "Multiline attribute is not separated by blank lines",
    code = 33,
    match_with = SyntaxKind::NODE_ATTR_SET
)]
struct MultilineSpacing;

impl Rule for MultilineSpacing {
    fn validate(&self, node: &SyntaxElement) -> Option<Report> {
        let NodeOrToken::Node(node) = node else {
            return None;
        };

        let attr_set = AttrSet::cast(node.clone())?;
        let assignments: Vec<_> = attr_set.entries().filter_map(utils::as_assignment).collect();

        // Each gap that needs opening, named by the attribute below it.
        // Two neighbouring multiline attributes describe the one gap between
        // them twice, once from each side, so the same attribute must not be
        // counted twice or the gap would end up two lines deep.
        let mut wanted: Vec<AttrpathValue> = Vec::new();
        let mut message = None;

        for (index, assignment) in assignments.iter().enumerate() {
            if !is_multiline_attribute(assignment) {
                continue;
            }

            let previous = index
                .checked_sub(1)
                .and_then(|before| assignments.get(before));

            if let Some(previous) = previous
                && !utils::blank_line_between(previous.syntax(), assignment.syntax())
            {
                note(&mut wanted, assignment);
                message.get_or_insert("a blank line should come before this multiline attribute");
            }

            if let Some(next) = assignments.get(index + 1)
                && !utils::blank_line_between(assignment.syntax(), next.syntax())
            {
                note(&mut wanted, next);
                message.get_or_insert("a blank line should follow the multiline attribute above");
            }
        }

        // Reported once for the set, as the ordering lints are. Every
        // missing blank line in it goes in together, since they share the
        // one rewritten set.
        let first = wanted.first()?;
        let at = first.syntax().text_range();
        let message = message?;

        let Some(spaced) = separate(node, &wanted) else {
            return Some(self.report().diagnostic(at, message));
        };

        Some(self.report().suggest(
            at,
            message,
            Suggestion::with_replacement(node.text_range(), spaced),
        ))
    }
}

/// Record an attribute wanting a blank line above it, once only.
fn note(wanted: &mut Vec<AttrpathValue>, assignment: &AttrpathValue) {
    if wanted
        .iter()
        .any(|existing| existing.syntax() == assignment.syntax())
    {
        return;
    }

    wanted.push(assignment.clone());
}

/// Rewrite the set with a blank line above each of `wanted`.
fn separate(node: &SyntaxNode, wanted: &[AttrpathValue]) -> Option<SyntaxNode> {
    let mut parts = utils::segments(node);

    for assignment in wanted {
        let position = parts
            .items
            .iter()
            .position(|item| &item.node == assignment.syntax())?;

        utils::add_blank_line_above(&mut parts, position);
    }

    let order = utils::unchanged_order(&parts);
    let text = utils::rebuild(&parts, &order);

    Root::parse(&text)
        .syntax()
        .descendants()
        .find_map(|candidate| AttrSet::cast(candidate).map(|set| set.syntax().clone()))
}

/// True when the attribute's value is written across more than one line.
///
/// A value merely wrapped onto the line below its `=` is not: the line break
/// sits before the value rather than inside it, so there is no block here to
/// give room to.
fn is_multiline_attribute(assignment: &AttrpathValue) -> bool {
    assignment
        .value()
        .is_some_and(|value| utils::is_multiline(value.syntax()))
}
