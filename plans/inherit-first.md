# New lint: `inherit_first` — `inherit` belongs at the top of a set

## Context

statix lints Nix antipatterns. On oliverdavies.uk's `nix/packages/default.nix` an
`inherit slugifier;` sits in the middle of the attrset between `spellcheck` and
`test`. The existing ordering lints deliberately leave `inherit` untouched
(`attribute_ordering` and `enable_first` both document that "an `inherit` …
stays where the author put it"), so nothing today nudges the author to move it.
A new lint is wanted: `inherit` entries belong at the top of an attrset, so
what the set borrows is seen before what it defines.

## Approach

A new lint `inherit_first`, code **38** (37 is taken by `collapsible_inherits`),
modelled on `enable_first` (report shape) and `inherit_blank_line` (entry
iteration, several-node kinds):

- `match_with = [SyntaxKind::NODE_ATTR_SET, SyntaxKind::NODE_LET_IN]`, same as
  `inherit_blank_line` — the "borrowed things first" argument applies to `let`
  bindings too.
- Iterate entries in order (`NODE_ATTRPATH_VALUE` / `NODE_INHERIT` children, as
  `inherit_blank_line`'s `is_entry` does, which excludes a `let`'s body).
- Report if an `inherit` appears anywhere after an assignment. One report per
  set, pointed at the first out-of-place `inherit` (pattern shared by
  `attribute_ordering` / `enable_first`), with a note-style message such as
  "`inherit` should come before the assignments in this set".
- Rewrite (like `enable_first`: fix when possible, diagnostic fallback):
  regenerate the node with `utils::segments` / `utils::rebuild`, moving every
  `inherit` position to the top, preserving the relative order of the inherits
  and of the assignments. Positions the inherits vacate keep their spacing;
  comments travel with the entry that precedes them, exactly as the existing
  rewrites do.

Small new util `inherit_positions(items)` in `lib/src/utils.rs` as the inverse
of the existing `assignment_positions`.

## Files to modify

- `lib/src/lints/inherit_first.rs` — new lint (docs comment, `#[lint]`,
  `Rule` impl, rewrite fn)
- `lib/src/utils.rs` — add `inherit_positions`
- `lib/src/lints.rs` — register `inherit_first` in the `lints!` macro
- `bin/tests/inherit_first.rs` — new snapshot tests (see below)

Docs in `readme.md` list only a subset of lints and are already stale
(missing `enable_first`, `attribute_ordering`, etc.); not touching them.

## Reuse

- `lib/src/lints/enable_first.rs` — report-once + `Suggestion::with_replacement`
  - reorder-within-positions pattern to copy.
- `lib/src/lints/inherit_blank_line.rs` — entry filtering (`is_entry`),
  two-kind `match_with`, message placement.
- `lib/src/utils.rs` — `segments`, `rebuild`, `unchanged_order`,
  `assignment_positions`, `is_multiline`.
- `bin/tests/attribute_ordering.rs` — snapshot test layout (`generate_tests!`
  with `indoc!` expressions); snapshots then generated with `cargo insta accept`.

## Steps

- [x] Add `inherit_positions` util
- [x] New `inherit_first` lint with rewrite; register in `lints.rs` (code 38)
- [x] Snapshot tests: covers cases below
- [x] Generate snapshots (`cargo insta accept`) and review the fixed output by eye

## Test cases

- inherit already at top — clean
- inherit after an assignment (the oliverdavies.uk shape) — reported, moved to top
- several inherits scattered among assignments — all moved, relative order kept
- inherit mid-set in a `let … in` binding
- single-line set — no report (no room; same skip as other rewrites)
- comments above entries move with their entry
- `rec` attrset survives the rewrite
- set with only inherits (trivially clean)

## Decisions (confirmed in review)

1. **Order vs `enable`**: `inherit` goes before the `enable` family. With the
   rewrites composing unchanged, `inherit_first` puts the inherits at the very
   top and `enable_first` keeps `enable` at the front of the remaining
   _assignments_, so `inherit` → `enable` → settings falls out naturally.
2. **Fix or diagnose only**: a full rewrite, as with the other ordering lints,
   with a bare diagnostic when the rewrite can't parse.

## Verification

- `cargo insta test -p statix-tests` inside `bin` (or `cargo test` there, then
  review/accept new snapshots), then `cargo clippy` / fmt as CI requires.
- Manual: paste the oliverdavies.uk attrset into a scratch file and run the
  built binary (`cargo run -p statix -- <file>` … per bin layout) to confirm
  the message, errfmt output, and that `statix fix --dry-run` produces
  `{ inherit slugifier; spellcheck = pkgs.callPackage …; test = …; }`.
