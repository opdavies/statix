use crate::{Metadata, Report, Rule, Suggestion};

use macros::lint;
use rnix::{NodeOrToken, Root, SyntaxElement, SyntaxKind};

/// ## What it does
/// Checks that no two blank lines follow one another, and that nothing at
/// all follows the last attribute or element of a set or list. Whitespace
/// inside a multiline string is not covered, since it is part of the string's
/// content rather than layout around it.
///
/// ## Why is this bad?
/// One blank line groups a block; more than one is an accident. Consecutive
/// blank lines show up when a block was deleted or moved, and a tidy file
/// has no reason to carry them. A blank line before a closing brace does the
/// opposite of grouping: it dangles at the end of the set, and the rule that
/// lets a block breathe never asked for room on its right.
///
/// ## Example
/// ```nix
/// {
///   foo = 1;
///
///
///   bar = 2;
/// }
/// ```
///
/// Collapse the run.
/// ```nix
/// {
///   foo = 1;
///
///   bar = 2;
/// }
/// ```
#[lint(
    name = "blank_lines",
    note = "Excess blank lines",
    code = 35,
    match_with = SyntaxKind::TOKEN_WHITESPACE
)]
struct BlankLines;

impl Rule for BlankLines {
    fn validate(&self, node: &SyntaxElement) -> Option<Report> {
        let NodeOrToken::Token(token) = node else {
            return None;
        };

        let closes_set = token
            .next_sibling_or_token()
            .is_some_and(|next| {
                matches!(
                    next.kind(),
                    SyntaxKind::TOKEN_R_BRACE | SyntaxKind::TOKEN_R_BRACK
                )
            });

        let text = token.text();

        let (collapsed, message) = if closes_set {
            (
                collapse_before_closer(text),
                "no blank line may come before the closing delimiter",
            )
        } else {
            (
                collapse(text),
                "no more than one blank line may follow one another",
            )
        };

        let collapsed = collapsed?;

        let at = token.text_range();

        Some(self.report().suggest(
            at,
            message,
            Suggestion::with_replacement(at, whitespace(&collapsed)),
        ))
    }
}

/// Collapse runs of three or more newlines to exactly two, removing any run
/// of two or more blank lines while leaving a single blank line in place.
/// Returns `None` when the token holds no such run.
fn collapse(text: &str) -> Option<String> {
    collapse_to(text, 2)
}

/// Collapse every run of two or more newlines to exactly one, removing any
/// blank line at all. The single newline is the one that already puts the
/// closing brace on its own line.
fn collapse_before_closer(text: &str) -> Option<String> {
    collapse_to(text, 1)
}

fn collapse_to(text: &str, keep: u8) -> Option<String> {
    let mut collapsed = String::with_capacity(text.len());
    let mut newlines = 0u8;
    let mut changed = false;

    for ch in text.chars() {
        if ch == '\n' {
            newlines += 1;
            if newlines <= keep {
                collapsed.push('\n');
            } else {
                changed = true;
            }
        } else {
            newlines = 0;
            collapsed.push(ch);
        }
    }

    changed.then_some(collapsed)
}

/// Rebuild the token with the collapsed text.
///
/// A whitespace token cannot be constructed directly, so the text is parsed
/// as whitespace in front of an identifier and the token read back out, the
/// way it appears at a line break in real source.
fn whitespace(text: &str) -> rnix::SyntaxElement {
    let root = Root::parse(&format!("{text}x"));

    root.syntax()
        .descendants_with_tokens()
        .find_map(|element| {
            if element.kind() == SyntaxKind::TOKEN_WHITESPACE {
                Some(element)
            } else {
                None
            }
        })
        .unwrap_or_else(|| panic!("no whitespace token in rendered text `{text:?}`"))
}