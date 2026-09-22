use crate::{Metadata, Report, Rule, Suggestion};

use macros::lint;
use rnix::{
    NodeOrToken, SyntaxElement, SyntaxKind, SyntaxNode,
    ast::{AttrSet, Entry, HasEntry as _, Lambda, Param},
};
use rowan::ast::AstNode as _;

/// ## What it does
/// Checks for an empty variadic pattern: `{...}`, in a function
/// argument.
///
/// ## Why is this bad?
/// An empty pattern accepts arguments but never uses them. When the
/// body is a plain attribute set, the lambda wrapper is unnecessary
/// noise. Remove it and expose the body directly.
///
/// ## Example
///
/// ```nix
/// {
///   foo = { ... }: {
///     services.irmaseal-pkg.enable = true;
///   };
/// }
/// ```
///
/// Remove the empty parameter and colon:
///
/// ```nix
/// {
///   foo = {
///     services.irmaseal-pkg.enable = true;
///   };
/// }
/// ```
#[lint(
    name = "empty_pattern",
    note = "Found empty pattern in function argument",
    code = 10,
    match_with = SyntaxKind::NODE_LAMBDA
)]
struct EmptyPattern;

impl Rule for EmptyPattern {
    fn validate(&self, node: &SyntaxElement) -> Option<Report> {
        let NodeOrToken::Node(node) = node else {
            return None;
        };

        let lambda_expr = Lambda::cast(node.clone())?;

        let Some(Param::Pattern(pattern)) = lambda_expr.param() else {
            return None;
        };

        // no patterns within `{ }`
        if pattern.pat_entries().count() != 0 {
            return None;
        }

        // pattern is not bound
        if pattern.pat_bind().is_some() {
            return None;
        }

        let body = lambda_expr.body()?;

        if is_module(body.syntax()) {
            return None;
        }

        Some(self.report().suggest(
            pattern.syntax().text_range(),
            "This pattern is empty, remove it",
            Suggestion::with_replacement(node.text_range(), body.syntax().clone()),
        ))
    }
}

fn is_module(body: &SyntaxNode) -> bool {
    let Some(attr_set) = AttrSet::cast(body.clone()) else {
        return false;
    };

    attr_set
        .entries()
        .filter_map(|e| {
            let Entry::AttrpathValue(attrpath_value) = e else {
                return None;
            };

            attrpath_value.attrpath()
        })
        .any(|k| k.to_string() == "imports")
}
