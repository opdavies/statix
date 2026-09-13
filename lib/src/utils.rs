use rnix::{
    NodeOrToken, SyntaxKind, SyntaxNode, TextRange,
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

/// One item of a set or a list, with the trivia that precedes it.
pub struct Segment {
    /// Whitespace before any comment, including whatever blank line
    /// separates this item from the one above. It describes the gap rather
    /// than the item, so a reordering leaves it where it is.
    pub spacing: String,

    /// The comments immediately above the item, and the indentation after
    /// them. These describe the item, so a reordering takes them with it.
    pub comments: String,

    pub node: SyntaxNode,
}

/// A node's items, with the trivia around them and the text that opens and
/// closes it.
pub struct Parts {
    /// Everything before the first item, delimiter included. Capturing it
    /// verbatim rather than assuming a `{` is what keeps `rec` on a
    /// recursive set: dropping it silently turns every reference between
    /// the set's own attributes into an undefined variable.
    pub opening: String,

    pub items: Vec<Segment>,

    /// Trivia after the last item, and the closing delimiter.
    pub closing: String,
}

/// Split a node into the items it contains and everything around them.
///
/// Separating the two kinds of trivia is what lets a reordering carry a
/// comment along with the thing it describes while leaving the blank lines
/// that group the set where the author put them. Moving all of it would drag
/// blank lines around; moving none of it would strand every comment above
/// whatever ends up in its position.
pub fn segments(parent: &SyntaxNode) -> Parts {
    let mut items: Vec<Segment> = Vec::new();
    let mut opening = String::new();
    let mut spacing = String::new();
    let mut comments = String::new();

    for child in parent.children_with_tokens() {
        match child {
            NodeOrToken::Token(token) => match token.kind() {
                SyntaxKind::TOKEN_COMMENT => comments.push_str(token.text()),
                SyntaxKind::TOKEN_WHITESPACE => {
                    // Once a comment has been seen, the rest of the run
                    // belongs with it rather than with the gap above.
                    if comments.is_empty() {
                        spacing.push_str(token.text());
                    } else {
                        comments.push_str(token.text());
                    }
                }
                // Any other token is part of the syntax holding the items,
                // such as `rec`, `{` or `[`. Before the first item it opens
                // the node; afterwards it closes it.
                _ => {
                    if items.is_empty() {
                        opening.push_str(&std::mem::take(&mut spacing));
                        opening.push_str(&std::mem::take(&mut comments));
                        opening.push_str(token.text());
                    } else {
                        spacing.push_str(token.text());
                    }
                }
            },
            NodeOrToken::Node(node) => items.push(Segment {
                spacing: std::mem::take(&mut spacing),
                comments: std::mem::take(&mut comments),
                node,
            }),
        }
    }

    Parts {
        opening,
        items,
        closing: spacing + &comments,
    }
}

/// Write the items back out in the given order, between the text that opened
/// and closed the node.
///
/// `order[position]` names the item that should end up at `position`. The
/// gap before each position stays with the position, while the comments
/// above an item travel with it.
pub fn rebuild(parts: &Parts, order: &[usize]) -> String {
    let mut text = parts.opening.clone();

    for (position, &source) in order.iter().enumerate() {
        text.push_str(&parts.items[position].spacing);
        text.push_str(&parts.items[source].comments);
        text.push_str(&parts.items[source].node.text().to_string());
    }

    text.push_str(&parts.closing);

    text
}

/// The positions holding an assignment. An `inherit` is a different kind of
/// entry and stays where the author put it.
pub fn assignment_positions(items: &[Segment]) -> Vec<usize> {
    items
        .iter()
        .enumerate()
        .filter(|(_, item)| item.node.kind() == SyntaxKind::NODE_ATTRPATH_VALUE)
        .map(|(position, _)| position)
        .collect()
}

/// The positions an alphabetical ordering may move, which is every
/// assignment other than the enable family, that being a group of its own at
/// the top of the set.
pub fn sortable_positions(items: &[Segment]) -> Vec<usize> {
    assignment_positions(items)
        .into_iter()
        .filter(|&position| {
            AttrpathValue::cast(items[position].node.clone())
                .and_then(|assignment| attribute_name(&assignment))
                .is_some_and(|name| !is_enable_family(&name))
        })
        .collect()
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
