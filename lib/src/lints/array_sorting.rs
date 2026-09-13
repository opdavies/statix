use crate::{Metadata, Report, Rule, Suggestion, utils};

use macros::lint;
use rnix::{
    NodeOrToken, Root, SyntaxElement, SyntaxKind, SyntaxNode,
    ast::{Apply, AttrpathValue, List},
};
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
/// is for attributes. Strings are compared to one another by their content,
/// since the delimiter around a string is the formatter's choice rather
/// than the author's.
///
/// Only arrays whose elements are each written on one line are considered.
/// Where an element spans lines it is some larger expression, and the order
/// of those is much more likely to matter.
///
/// An array whose order is part of what it means is left alone: one
/// assigned to an attribute named `order`, and one passed to a function
/// that reads it against another by position, such as
/// `builtins.replaceStrings`.
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

/// Functions that read an array against another by position, so that
/// sorting one of them alone would leave the pair naming different things.
/// `builtins.replaceStrings` takes what to replace and what to replace it
/// with as two arrays matched by index.
static PAIRED_ARGUMENTS: &[&str] = &[
    "builtins.replaceStrings",
    "lib.replaceStrings",
    "replaceStrings",
];

impl Rule for ArraySorting {
    fn validate(&self, node: &SyntaxElement) -> Option<Report> {
        let NodeOrToken::Node(node) = node else {
            return None;
        };

        if !utils::is_multiline(node) || is_ordered(node) {
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

        let out_of_place = parts
            .items
            .windows(2)
            .find(|pair| sort_key(&pair[0].node) > sort_key(&pair[1].node))
            .map(|pair| pair[1].node.clone())?;

        let mut order: Vec<usize> = (0..parts.items.len()).collect();
        order.sort_by_key(|&position| sort_key(&parts.items[position].node));

        let text = utils::rebuild(&parts, &order);

        let sorted = Root::parse(&text)
            .syntax()
            .descendants()
            .find_map(|node| List::cast(node).map(|list| list.syntax().clone()))?;

        Some(self.report().suggest(
            out_of_place.text_range(),
            format!("`{}` is out of alphabetical order", out_of_place.text()),
            Suggestion::with_replacement(node.text_range(), sorted),
        ))
    }
}

/// The text an element is compared by.
///
/// A string is compared as though it carried the same delimiter as every
/// other string, because which one it actually carries is the formatter's
/// decision rather than the author's: nixfmt writes a single-line string
/// as `"..."` unless it contains a double quote, which forces `''...''`.
/// Comparing the source text would order by that delimiter, `"` sorting
/// before `'`, and so separate the two forms into groups instead of
/// ordering the strings themselves.
///
/// The delimiter is replaced rather than removed, which is what keeps a
/// string sorting against a neighbouring expression where it did before.
/// Both `"` and `'` already sort before the `(` of a parenthesised
/// expression and the letter beginning an identifier, so a list mixing a
/// string with either is left exactly as it was.
fn sort_key(node: &SyntaxNode) -> String {
    let text = node.text().to_string();

    if node.kind() != SyntaxKind::NODE_STRING {
        return text;
    }

    for delimiter in ["''", "\""] {
        if let Some(content) = text
            .strip_prefix(delimiter)
            .and_then(|rest| rest.strip_suffix(delimiter))
        {
            return format!("\"{content}");
        }
    }

    text
}

/// True when the order of the array is part of what it means, so that
/// sorting it would be a change to the code rather than to its layout.
fn is_ordered(list: &SyntaxNode) -> bool {
    assigned_name(list).is_some_and(|name| name == "order")
        || application_head(list).is_some_and(|head| PAIRED_ARGUMENTS.contains(&head.as_str()))
}

/// The name the array is assigned to.
///
/// The last attribute of a dotted path is the one naming the array itself,
/// so `text.readme.order` is `order`.
fn assigned_name(list: &SyntaxNode) -> Option<String> {
    let assignment = AttrpathValue::cast(list.parent()?)?;
    let name = assignment.attrpath()?.attrs().last()?;

    Some(name.syntax().text().to_string())
}

/// The function an array is an argument of.
///
/// An application takes one argument at a time, so the second array of
/// `f [ ... ] [ ... ]` is an argument of `f [ ... ]` rather than of `f`.
/// Walking to the head of the chain names `f` for both of them.
fn application_head(list: &SyntaxNode) -> Option<String> {
    let mut head = Apply::cast(list.parent()?)?.lambda()?;

    while let Some(inner) = Apply::cast(head.syntax().clone()) {
        head = inner.lambda()?;
    }

    Some(head.syntax().text().to_string())
}
