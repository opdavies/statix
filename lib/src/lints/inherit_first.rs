use crate::{Metadata, Report, Rule, Suggestion, utils};

use macros::lint;
use rnix::{NodeOrToken, Root, SyntaxElement, SyntaxKind, SyntaxNode};

/// ## What it does
/// Checks that the `inherit` statements of a set or a `let` binding come
/// before the attributes, not after or between them.
///
/// ## Why is this bad?
/// An `inherit` is a different kind of entry from an assignment: it names
/// things the set borrows rather than the value those things have. Read in
/// the order it is written, a set announces what it takes before what it
/// adds, so anything the reader might wonder about the origin of is cleared
/// up first.
///
/// The attribute ordering lints leave an `inherit` wherever the author put
/// it, since reordering attributes past it would be judged by a different
/// rule: `enable_first` puts `enable` at the front of the assignments, which
/// means after the inherits, and `attribute_ordering` sorts what remains.
///
/// ## Example
/// ```nix
/// {
///   spellcheck = pkgs.callPackage ./spellcheck.pkg.nix { };
///   inherit slugifier;
///   test = pkgs.callPackage ./test.pkg.nix { };
/// }
/// ```
///
/// Put the inherit first.
/// ```nix
/// {
///   inherit slugifier;
///
///   spellcheck = pkgs.callPackage ./spellcheck.pkg.nix { };
///   test = pkgs.callPackage ./test.pkg.nix { };
/// }
/// ```
#[lint(
    name = "inherit_first",
    note = "`inherit` is not at the top of the set",
    code = 38,
    match_with = [
        SyntaxKind::NODE_ATTR_SET,
        SyntaxKind::NODE_LET_IN
    ]
)]
struct InheritFirst;

impl Rule for InheritFirst {
    fn validate(&self, node: &SyntaxElement) -> Option<Report> {
        let NodeOrToken::Node(node) = node else {
            return None;
        };

        // A set or binding written inline has no reading order to speak of.
        if !utils::is_multiline(node) {
            return None;
        }

        let entries: Vec<SyntaxNode> = node
            .children()
            .filter(|child| is_entry(child.kind()))
            .collect();

        // The first `inherit` written after an assignment. Everything the
        // rewrite needs to know follows from the whole set being reordered
        // rather than from this one entry.
        let mut seen_assignment = false;
        let misplaced = entries
            .iter()
            .find(|entry| match entry.kind() {
                SyntaxKind::NODE_ATTRPATH_VALUE => {
                    seen_assignment = true;
                    false
                }
                _ => seen_assignment,
            })
            .cloned()?;

        let at = misplaced.text_range();
        let message = "`inherit` should come before the assignments above it";

        // The report does not depend on the rewrite succeeding. Were it to,
        // a set this could not rewrite would go unreported rather than
        // merely unfixed.
        let Some(moved) = move_inherits_first(node) else {
            return Some(self.report().diagnostic(at, message));
        };

        Some(self.report().suggest(
            at,
            message,
            Suggestion::with_replacement(node.text_range(), moved),
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

/// Rewrite the set with the `inherit` statements at the top.
///
/// The positions the entries already occupy are filled with the inherits
/// first and the assignments after, each group keeping its relative order.
/// A `let` body is neither and stays where it was. Each inherit takes the
/// gap belonging to the position it lands in, while the comments above it
/// travel with it.
fn move_inherits_first(node: &SyntaxNode) -> Option<SyntaxNode> {
    let parts = utils::segments(node);

    let positions: Vec<usize> = (0..parts.items.len())
        .filter(|&position| is_entry(parts.items[position].node.kind()))
        .collect();

    let mut sources = utils::inherit_positions(&parts.items);
    sources.extend(utils::assignment_positions(&parts.items));

    let mut order: Vec<usize> = (0..parts.items.len()).collect();

    for (&target, &source) in positions.iter().zip(&sources) {
        order[target] = source;
    }

    let text = utils::rebuild(&parts, &order);

    Root::parse(&text)
        .syntax()
        .descendants()
        .find_map(|candidate| (candidate.kind() == node.kind()).then_some(candidate.clone()))
}
