use rnix::{
    SyntaxKind, SyntaxNode, TextRange,
    ast::{Attr, AttrpathValue, Entry},
};
use rowan::ast::AstNode as _;

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
