use crate::{Metadata, Report, Rule, utils};

use macros::lint;
use rnix::{
    NodeOrToken, SyntaxElement, SyntaxKind,
    ast::{AttrSet, AttrpathValue, Expr, HasEntry as _},
};
use rowan::ast::AstNode as _;

/// ## What it does
/// Checks that an attribute holding a multiline array is separated from the
/// attributes around it by a blank line.
///
/// ## Why is this bad?
/// A multiline array is a block rather than a one-line setting, and reads as
/// one when it is given room. Without the blank lines its first and last
/// entries crowd against unrelated attributes.
///
/// The blank line belongs before the attribute, not between its `=` and its
/// `[`, since nothing can sit there.
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
    name = "array_spacing",
    note = "Multiline array is not separated by blank lines",
    code = 33,
    match_with = SyntaxKind::NODE_ATTR_SET
)]
struct ArraySpacing;

impl Rule for ArraySpacing {
    fn validate(&self, node: &SyntaxElement) -> Option<Report> {
        let NodeOrToken::Node(node) = node else {
            return None;
        };

        let attr_set = AttrSet::cast(node.clone())?;
        let assignments: Vec<_> = attr_set.entries().filter_map(utils::as_assignment).collect();

        let mut report = self.report();
        let mut found = false;

        for (index, assignment) in assignments.iter().enumerate() {
            if !holds_multiline_array(assignment) {
                continue;
            }

            let previous = index.checked_sub(1).and_then(|before| assignments.get(before));

            if let Some(previous) = previous
                && !utils::blank_line_between(previous.syntax(), assignment.syntax())
            {
                report = report.diagnostic(
                    assignment.syntax().text_range(),
                    "a blank line should come before this multiline array",
                );
                found = true;
            }

            if let Some(next) = assignments.get(index + 1)
                && !utils::blank_line_between(assignment.syntax(), next.syntax())
            {
                report = report.diagnostic(
                    next.syntax().text_range(),
                    "a blank line should follow the multiline array above",
                );
                found = true;
            }
        }

        found.then_some(report)
    }
}

fn holds_multiline_array(assignment: &AttrpathValue) -> bool {
    let Some(Expr::List(list)) = assignment.value() else {
        return false;
    };

    utils::is_multiline(list.syntax())
}
