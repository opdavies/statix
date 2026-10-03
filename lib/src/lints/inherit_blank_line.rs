use crate::{Metadata, Report, Rule, Suggestion, utils};

use macros::lint;
use rnix::{NodeOrToken, Root, SyntaxElement, SyntaxKind, SyntaxNode};

/// ## What it does
/// Checks that a blank line follows each `inherit` statement in a set or a
/// `let` binding, separating it from the entries that come after.
///
/// ## Why is this bad?
/// An `inherit` is a different kind of entry from an assignment: it names
/// things it borrows rather than the value those things have. The blank line
/// is what signs that change of subject, the way one is written between the
/// paragraphs of a page.
///
/// Several `inherit` statements in a row form one group, so only the last of
/// them is checked. An `inherit` last in its set has nothing after it to be
/// separated from.
///
/// The neighbouring `collapsible_inherits` lint is what has something to say
/// about neighbouring inherits: they are better written as one statement than
/// as a group separated by blank lines.
///
/// ## Example
/// ```nix
/// {
///   inputs = mkInputs {
///     inherit system;
///     owner = "me";
///   };
/// }
/// ```
///
/// Separate the inherit from the rest.
/// ```nix
/// {
///   inputs = mkInputs {
///     inherit system;
///
///     owner = "me";
///   };
/// }
/// ```
#[lint(
    name = "inherit_blank_line",
    note = "No blank line after `inherit`",
    code = 36,
    match_with = [
        SyntaxKind::NODE_ATTR_SET,
        SyntaxKind::NODE_LET_IN
    ]
)]
struct InheritBlankLine;

impl Rule for InheritBlankLine {
    fn validate(&self, node: &SyntaxElement) -> Option<Report> {
        let NodeOrToken::Node(node) = node else {
            return None;
        };

        // A set or binding written inline has nowhere to put a blank line.
        if !utils::is_multiline(node) {
            return None;
        }

        let entries: Vec<SyntaxNode> = node
            .children()
            .filter(|child| is_entry(child.kind()))
            .collect();

        // Each pair of neighbouring entries where the upper one is an
        // `inherit` and no blank line sits between them. Both the entries are
        // collected before any check, so the group a run of inherits forms
        // needs no special case: only the pair reaching past it reports.
        let mut wanted: Vec<SyntaxNode> = Vec::new();
        let mut message = None;

        for pair in entries.windows(2) {
            let (above, below) = (&pair[0], &pair[1]);

            if above.kind() != SyntaxKind::NODE_INHERIT
                // Inherits written one after another form a group rather
                // than something to separate, and `collapsible_inherits` is
                // what has something to say about them.
                || below.kind() == SyntaxKind::NODE_INHERIT
                || utils::blank_line_between(above, below)
            {
                continue;
            }

            note(&mut wanted, below);
            message.get_or_insert("a blank line should follow the `inherit` above");
        }

        // Reported once for the set, as the ordering lints are. Every
        // missing blank line in it goes in together, since they share the
        // one rewritten set.
        let first = wanted.first()?;
        let at = first.text_range();
        let message = message?;

        let Some(separated) = separate(node, &wanted) else {
            return Some(self.report().diagnostic(at, message));
        };

        Some(self.report().suggest(
            at,
            message,
            Suggestion::with_replacement(node.text_range(), separated),
        ))
    }
}

/// True for the two kinds of entry a set or a binding may hold. The body of
/// a `let` is a child node too, but of another kind.
fn is_entry(kind: SyntaxKind) -> bool {
    matches!(
        kind,
        SyntaxKind::NODE_ATTRPATH_VALUE | SyntaxKind::NODE_INHERIT
    )
}

/// Record an entry wanting a blank line above it, once only.
fn note(wanted: &mut Vec<SyntaxNode>, below: &SyntaxNode) {
    if wanted.iter().any(|existing| existing == below) {
        return;
    }

    wanted.push(below.clone());
}

/// Rewrite the set with a blank line above each of `wanted`.
fn separate(node: &SyntaxNode, wanted: &[SyntaxNode]) -> Option<SyntaxNode> {
    let mut parts = utils::segments(node);

    for below in wanted {
        let position = parts.items.iter().position(|item| &item.node == below)?;

        utils::add_blank_line_above(&mut parts, position);
    }

    let order = utils::unchanged_order(&parts);
    let text = utils::rebuild(&parts, &order);

    Root::parse(&text)
        .syntax()
        .descendants()
        .find_map(|candidate| (candidate.kind() == node.kind()).then_some(candidate))
}
