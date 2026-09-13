use crate::{Metadata, Report, Rule};

use macros::lint;
use rnix::{
    NodeOrToken, SyntaxElement, SyntaxKind,
    ast::{Attr, AttrSet, AttrpathValue, Entry, HasEntry as _},
};
use rowan::ast::AstNode as _;

/// ## What it does
/// Checks that `enable` is the first attribute in an attribute set.
///
/// ## Why is this bad?
/// Whether a block is switched on at all is the first thing a reader needs
/// to know, so it reads better at the top than buried among the settings it
/// governs.
///
/// ## Example
/// ```nix
/// {
///   nssmdns4 = true;
///   openFirewall = true;
///   enable = true;
/// }
/// ```
///
/// Put `enable` first.
/// ```nix
/// {
///   enable = true;
///
///   nssmdns4 = true;
///   openFirewall = true;
/// }
/// ```
#[lint(
    name = "enable_first",
    note = "`enable` is not the first attribute",
    code = 30,
    match_with = SyntaxKind::NODE_ATTR_SET
)]
struct EnableFirst;

impl Rule for EnableFirst {
    fn validate(&self, node: &SyntaxElement) -> Option<Report> {
        let NodeOrToken::Node(node) = node else {
            return None;
        };

        let attr_set = AttrSet::cast(node.clone())?;
        let mut assignments = attr_set.entries().filter_map(as_assignment);

        // An `inherit` takes no part in this. Only the assignments are
        // ordered, so the first of those is the position `enable` wants.
        let first = assignments.next()?;

        if is_enable(&first) {
            return None;
        }

        let enable = assignments.find(is_enable)?;

        Some(self.report().diagnostic(
            enable.syntax().text_range(),
            "`enable` should be the first attribute in this set",
        ))
    }
}

fn as_assignment(entry: Entry) -> Option<AttrpathValue> {
    match entry {
        Entry::AttrpathValue(attrpath_value) => Some(attrpath_value),
        Entry::Inherit(_) => None,
    }
}

/// True when the attribute is named exactly `enable`.
///
/// A dotted path such as `syntaxHighlighting.enable` belongs to the set it
/// names rather than this one, and `enableCompletion` is its own attribute,
/// so neither counts.
fn is_enable(attrpath_value: &AttrpathValue) -> bool {
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
