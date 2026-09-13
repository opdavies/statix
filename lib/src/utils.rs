use rnix::{
    SyntaxKind, SyntaxNode, TextRange,
    ast::{Attr, AttrpathValue, Entry},
};
use rowan::{Direction, ast::AstNode as _};

/// The name of an attribute, as written.
///
/// Reading the attrpath rather than splitting the node's text on `=` keeps
/// this correct for values containing one, and for quoted keys such as
/// `":${toString port}"`. An attrpath may span lines, so its whitespace is
/// collapsed to keep the name on one line.
pub fn attribute_name(attrpath_value: &AttrpathValue) -> Option<String> {
    let attrpath = attrpath_value.attrpath()?;

    Some(
        attrpath
            .syntax()
            .text()
            .to_string()
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" "),
    )
}

/// The assignment in an entry, discarding `inherit`.
pub fn as_assignment(entry: Entry) -> Option<AttrpathValue> {
    match entry {
        Entry::AttrpathValue(attrpath_value) => Some(attrpath_value),
        Entry::Inherit(_) => None,
    }
}

/// True when the attribute is named exactly `enable`.
///
/// A dotted path such as `syntaxHighlighting.enable` belongs to the set it
/// names rather than this one.
pub fn is_enable(attrpath_value: &AttrpathValue) -> bool {
    let Some(attrpath) = attrpath_value.attrpath() else {
        return false;
    };

    let mut attrs = attrpath.attrs();

    let Some(Attr::Ident(ident)) = attrs.next() else {
        return false;
    };

    if attrs.next().is_some() {
        return false;
    }

    ident.to_string() == "enable"
}

/// True for `enable` and its camelCase relatives, such as `enableCompletion`.
///
/// These are written as one group at the top of a set, so they take no part
/// in the alphabetical ordering applied to everything else. Requiring an
/// uppercase letter after the prefix keeps `enabledCollectors` out of it,
/// that being the word "enabled" rather than an `enable` toggle.
pub fn is_enable_family(name: &str) -> bool {
    let Some(rest) = name.strip_prefix("enable") else {
        return false;
    };

    rest.chars()
        .next()
        .is_none_or(|first| first.is_ascii_uppercase())
}

/// True when a blank line separates two sibling nodes.
///
/// Any comment sitting between them is stepped over, so a blank line above
/// the comment still counts as separating the two. Two newlines in one run
/// of whitespace is what leaves an empty line between them.
pub fn blank_line_between(first: &SyntaxNode, second: &SyntaxNode) -> bool {
    first
        .siblings_with_tokens(Direction::Next)
        .take_while(|element| element.as_node() != Some(second))
        .filter_map(|element| element.into_token())
        .filter(|token| token.kind() == SyntaxKind::TOKEN_WHITESPACE)
        .any(|token| token.text().matches('\n').count() >= 2)
}

/// True when the node is written across more than one line.
pub fn is_multiline(node: &SyntaxNode) -> bool {
    node.text().to_string().contains('\n')
}

pub fn with_preceeding_whitespace(node: &SyntaxNode) -> TextRange {
    let start = node.prev_sibling_or_token().map_or_else(
        || node.text_range().start(),
        |t| {
            if t.kind() == SyntaxKind::TOKEN_WHITESPACE {
                t.text_range().start()
            } else {
                t.text_range().end()
            }
        },
    );
    let end = node.text_range().end();
    TextRange::new(start, end)
}
