use crate::{Metadata, Report, Rule, Suggestion, utils};

use macros::lint;
use rnix::{
    NodeOrToken, Root, SyntaxElement, SyntaxKind, SyntaxNode, SyntaxToken, TextRange, ast::Inherit,
};
use rowan::{Direction, ast::AstNode as _};

/// ## What it does
/// Checks for consecutive `inherit` statements that could be written as one.
///
/// ## Why is this bad?
/// Several statements of the same source read as an accident of editing
/// rather than an intention. When the statements agree on where they inherit
/// from, one statement takes all of the names.
///
/// Two statements are merged only when both carry the same `from` source, or
/// neither does. A blank line or comment between them, or inside the second
/// statement, says the author meant to keep them apart, and the run is left
/// as it is. Statements of differing sources, such as
/// `inherit (pkgs) foo;` and `inherit lib;`, are distinct and untouched.
///
/// This is reported from the head of the run, so that two statements find the
/// lint once rather than the second and third of them each raising their own
/// finding.
///
/// ## Example
///
/// ```nix
/// inherit (pkgs) foo;
/// inherit (pkgs) bar;
/// ```
///
/// Merge them.
///
/// ```nix
/// inherit (pkgs) foo bar;
/// ```
#[lint(
    name = "collapsible_inherits",
    note = "Consecutive inherit statements are collapsible",
    code = 37,
    match_with = SyntaxKind::NODE_INHERIT
)]
struct CollapsibleInherits;

impl Rule for CollapsibleInherits {
    fn validate(&self, node: &SyntaxElement) -> Option<Report> {
        let NodeOrToken::Node(node) = node else {
            return None;
        };

        let inherit_stmt = Inherit::cast(node.clone())?;
        let run = run_of_inherits(&inherit_stmt)?;
        let last = run.last()?.syntax().text_range();

        let attrs: Vec<String> = run
            .iter()
            .flat_map(Inherit::attrs)
            .map(|attr| attr.syntax().text().to_string())
            .collect();

        // Nothing inherited between the statements means there is nothing to
        // merge; the empty `inherit` statements themselves have their own
        // lint.
        if attrs.is_empty() {
            return None;
        }

        let Some(merged) = merge(&run, &attrs) else {
            return Some(self.report().diagnostic(
                last,
                "These consecutive `inherit` statements can be merged into one",
            ));
        };

        let replacement_at = TextRange::new(
            run[0].syntax().text_range().start(),
            run.last()?.syntax().text_range().end(),
        );

        Some(self.report().suggest(
            last,
            "These consecutive `inherit` statements can be merged into one",
            Suggestion::with_replacement(replacement_at, merged.syntax().clone()),
        ))
    }
}

/// The statement itself and every `inherit` written directly on from it and
/// carrying the same `from` source. `None` when the statement has another
/// `inherit` above it, for then it is not the head of its run, or when the
/// next statement differs on where it inherits from, for the run ends there.
fn run_of_inherits(head: &Inherit) -> Option<Vec<Inherit>> {
    let previous = neighbour(head.syntax(), Direction::Prev)?;

    if previous
        .as_node()
        .is_some_and(|node| node.kind() == SyntaxKind::NODE_INHERIT)
    {
        return None;
    }

    let mut run = vec![head.clone()];

    while let Some(next) = neighbour(run.last()?.syntax(), Direction::Next) {
        let Some(node) = next.as_node() else {
            break;
        };

        let Some(candidate) = Inherit::cast(node.clone()) else {
            break;
        };

        // Trivia of substance splits a run: a blank line in the gap, a comment in
        // the gap, or a comment inside the candidate says the author meant to
        // keep the statements apart.
        if utils::blank_line_between(run.last()?.syntax(), node)
            || gap_has_comment(run.last()?.syntax(), node)
            || node
                .descendants_with_tokens()
                .any(|element| element.kind() == SyntaxKind::TOKEN_COMMENT)
        {
            break;
        }

        if same_source(&run[0], &candidate) {
            run.push(candidate);
        } else {
            break;
        }
    }

    (run.len() > 1).then_some(run)
}

/// The element immediately past one, stepping over every run of whitespace
/// and any comments.
///
/// A candidate other than an entry, such as the `}` closing a set, comes back
/// as a token and says the set holds no further inherit statements.
fn neighbour(
    node: &SyntaxNode,
    direction: Direction,
) -> Option<NodeOrToken<SyntaxNode, SyntaxToken>> {
    node.siblings_with_tokens(direction)
        .skip(1)
        .find(|element| {
            !matches!(
                element.kind(),
                SyntaxKind::TOKEN_WHITESPACE | SyntaxKind::TOKEN_COMMENT
            )
        })
}

/// True when a comment sits in the gap between two neighbouring entries.
fn gap_has_comment(from: &SyntaxNode, to: &SyntaxNode) -> bool {
    from.siblings_with_tokens(Direction::Next)
        .skip(1)
        .take_while(|element| element.as_node() != Some(to))
        .any(|element| element.kind() == SyntaxKind::TOKEN_COMMENT)
}

/// True when the two statements inherit from the same source, which is when
/// both carry none, or both carry one and it textually matches.
fn same_source(first: &Inherit, second: &Inherit) -> bool {
    match (first.from(), second.from()) {
        (None, None) => true,
        (Some(first), Some(second)) => first.syntax().text() == second.syntax().text(),
        _ => false,
    }
}

/// Build the single statement carrying every inherited attribute of the run.
///
/// A statement of the wanted shape cannot be written into a token directly,
/// so the text is parsed where it belongs, inside a set, and the `inherit`
/// node read back out.
fn merge(run: &[Inherit], attrs: &[String]) -> Option<Inherit> {
    let first = run.first()?;

    // The source travels with the statement, the semicolon does not: the
    // attributes of the whole run go on ahead of it, joined into the one
    // list.
    let from = first
        .from()
        .map_or_else(String::new, |source| format!(" {}", source.syntax().text()));

    let text = format!("{{ inherit{from} {}; }}", attrs.join(" "));

    Root::parse(&text)
        .syntax()
        .descendants()
        .find_map(Inherit::cast)
}
