use crate::{Metadata, Report, Rule, Suggestion, utils};

use macros::lint;
use rnix::{NodeOrToken, Root, SyntaxElement, SyntaxKind, SyntaxNode, ast::Inherit};
use rowan::ast::AstNode as _;

/// ## What it does
/// Checks that the names of an `inherit` statement are in alphabetical
/// order.
///
/// ## Why is this bad?
/// An `inherit` borrows things rather than defining them, so its list reads
/// like a contents page: one that is ordered by nothing in particular gives
/// a reader no way to find a name except to read all of them, and gives two
/// people editing it no agreed place to add one.
///
/// Ordering is by byte, which is what `:sort` in vim and `LC_ALL=C sort`
/// both produce. Uppercase therefore sorts before lowercase, as it does for
/// the attributes judged by `attribute_ordering` and the elements judged by
/// `array_sorting`; the locale-aware ordering of a bare `sort` is
/// deliberately not used, since it would have the lint disagree with itself
/// across machines.
///
/// Every name takes part in the ordering, quoted and `${…}` names
/// alongside plain identifiers, each compared by the text it is written
/// with. The expression an `inherit (…) …` borrows from is not a name and
/// stays where it is.
///
/// This checks each statement alone. What has to say about inherits of
/// different sets, or of several statements run together, is said by
/// `inherit_first` and `collapsible_inherits`.
///
/// ## Example
/// ```nix
/// inherit (lib)
///   mkEnableOption
///   mkIf
///   mkMerge
///   mkPackageOption
///   mkOption
///   ;
/// ```
///
/// Sort the names.
/// ```nix
/// inherit (lib)
///   mkEnableOption
///   mkIf
///   mkMerge
///   mkOption
///   mkPackageOption
///   ;
/// ```
#[lint(
    name = "inherit_ordering",
    note = "Inherited names are not in alphabetical order",
    code = 39,
    match_with = SyntaxKind::NODE_INHERIT
)]
struct InheritOrdering;

impl Rule for InheritOrdering {
    fn validate(&self, node: &SyntaxElement) -> Option<Report> {
        let NodeOrToken::Node(node) = node else {
            return None;
        };

        let parts = utils::segments(node);

        // The names of the statement, in the order written. A single name
        // is in order by definition; the expression an `inherit (…) …`
        // borrows from makes another item of one, so the counting is of
        // names rather than of items.
        let names: Vec<&utils::Segment> = utils::inherit_name_positions(&parts.items)
            .into_iter()
            .filter_map(|position| parts.items.get(position))
            .collect();

        let out_of_place = names
            .windows(2)
            .find(|pair| sort_key(&pair[0].node) > sort_key(&pair[1].node))
            .map(|pair| pair[1])?;

        let name = sort_key(&out_of_place.node);

        let at = out_of_place.node.text_range();
        let message = format!("`{name}` is out of alphabetical order");

        // The report does not depend on the rewrite succeeding. Were it to,
        // a statement this could not rewrite would go unreported rather than
        // merely unfixed.
        let Some(sorted) = sort_names(node) else {
            return Some(self.report().diagnostic(at, message));
        };

        Some(self.report().suggest(
            at,
            message,
            Suggestion::with_replacement(node.text_range(), sorted),
        ))
    }
}

/// Rewrite the statement with its names in order.
///
/// Only the positions the names already occupy are reordered among
/// themselves, so the expression an `inherit (…) …` borrows from keeps its
/// place at the front. Each name keeps the trivia that preceded it, so a
/// comment above one moves with it.
fn sort_names(node: &SyntaxNode) -> Option<SyntaxNode> {
    let parts = utils::segments(node);
    let sortable = utils::inherit_name_positions(&parts.items);

    let mut sorted = sortable.clone();
    sorted.sort_by_key(|&position| sort_key(&parts.items[position].node));

    let mut order: Vec<usize> = (0..parts.items.len()).collect();

    for (&target, &source) in sortable.iter().zip(&sorted) {
        order[target] = source;
    }

    let text = utils::rebuild(&parts, &order);

    // The statement's own text is not valid Nix on its own — an `inherit`
    // belongs inside a set or a binding — so the rewrite is re-parsed
    // inside braces that the suggestion never carries.
    Root::parse(&format!("{{ {text} }}"))
        .syntax()
        .descendants()
        .find_map(|candidate| Inherit::cast(candidate).map(|inherit| inherit.syntax().clone()))
}

/// Inherit names are compared by the text they are written with.
///
/// A quoted or `${…}` name needs no normalisation the way a string element
/// of an array does: an `inherit` holds only names, not expressions, and
/// nothing delimits them that a formatter would have chosen instead.
fn sort_key(name: &SyntaxNode) -> String {
    name.text().to_string()
}
