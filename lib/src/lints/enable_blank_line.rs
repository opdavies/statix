use crate::{Metadata, Report, Rule, Suggestion, utils};

use macros::lint;
use rnix::{
    NodeOrToken, Root, SyntaxElement, SyntaxKind, SyntaxNode,
    ast::{AttrSet, HasEntry as _},
};
use rowan::ast::AstNode as _;

/// ## What it does
/// Checks that a blank line follows `enable` and its camelCase relatives,
/// separating them from the settings they govern.
///
/// ## Why is this bad?
/// Whether a block is switched on is a different question from how it is
/// configured, and the blank line is what says so at a glance.
///
/// The family is written as one group, so no blank line falls between its
/// members. The one that matters comes after the last of them.
///
/// ## Example
/// ```nix
/// {
///   enable = true;
///   enableCompletion = true;
///   cdpath = "a";
/// }
/// ```
///
/// Separate the group from the rest.
/// ```nix
/// {
///   enable = true;
///   enableCompletion = true;
///
///   cdpath = "a";
/// }
/// ```
#[lint(
    name = "enable_blank_line",
    note = "No blank line after `enable`",
    code = 32,
    match_with = SyntaxKind::NODE_ATTR_SET
)]
struct EnableBlankLine;

impl Rule for EnableBlankLine {
    fn validate(&self, node: &SyntaxElement) -> Option<Report> {
        let NodeOrToken::Node(node) = node else {
            return None;
        };

        let attr_set = AttrSet::cast(node.clone())?;

        // A set written inline has nowhere to put a blank line.
        if !utils::is_multiline(node) {
            return None;
        }

        let assignments: Vec<_> = attr_set
            .entries()
            .filter_map(utils::as_assignment)
            .collect();

        let is_family = |assignment: &_| {
            utils::attribute_name(assignment).is_some_and(|name| utils::is_enable_family(&name))
        };

        // The group runs from the top of the set. Where `enable` sits
        // somewhere further down, it is `enable_first` that has something to
        // say, not this.
        let after_group = assignments.iter().position(|a| !is_family(a))?;

        if after_group == 0 {
            return None;
        }

        let last_of_group = &assignments[after_group - 1];
        let next = &assignments[after_group];

        if utils::blank_line_between(last_of_group.syntax(), next.syntax()) {
            return None;
        }

        let at = last_of_group.syntax().text_range();
        let message = "a blank line should separate this from what follows";

        let Some(separated) = separate(node, next.syntax()) else {
            return Some(self.report().diagnostic(at, message));
        };

        Some(self.report().suggest(
            at,
            message,
            Suggestion::with_replacement(node.text_range(), separated),
        ))
    }
}

/// Rewrite the set with a blank line above `next`.
fn separate(node: &SyntaxNode, next: &SyntaxNode) -> Option<SyntaxNode> {
    let mut parts = utils::segments(node);

    let position = parts.items.iter().position(|item| &item.node == next)?;

    utils::add_blank_line_above(&mut parts, position);

    let order = utils::unchanged_order(&parts);
    let text = utils::rebuild(&parts, &order);

    Root::parse(&text)
        .syntax()
        .descendants()
        .find_map(|candidate| AttrSet::cast(candidate).map(|set| set.syntax().clone()))
}
