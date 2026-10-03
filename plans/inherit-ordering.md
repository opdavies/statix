# New lint: `inherit_ordering` — names in an `inherit` are sorted alphabetically

## Context

Follow-up to `plans/inherit-first.md` (implemented). The user pasted:

```nix
inherit (lib)
  mkEnableOption
  mkIf
  mkMerge
  mkPackageOption
  mkOption
  ;
```

and wants the inherited names sorted alphabetically (`mkOption` must come
before `mkPackageOption`). statix already judges ordering elsewhere
(`attribute_ordering`, `array_sorting`), but nothing checks the names listed
inside an `inherit` statement — including the names taken from an
`inherit (expr)`.

## Approach

A new lint on `SyntaxKind::NODE_INHERIT`, code **39** (38 is taken by
`inherit_first`), modelled on `array_sorting` (sort-the-items shape) with
report style borrowed from `attribute_ordering`.

- **Match**: `match_with = SyntaxKind::NODE_INHERIT`.
- **Scope**: sort the _names within_ each `inherit` statement only, of every
  inherit regardless of shape. The from-expression of `inherit (lib) …`
  stays where it is. Cross-statement ordering (e.g. two `inherit (x) …`
  groups) is left to `inherit_first` / `collapsible_inherits`.
- **No multiline gate**: single-line inherits are judged too — the user
  confirmed this differs from `array_sorting`, which skips inline arrays.
- **Ordering**: by byte, matching `attribute_ordering` (`:sort` /
  `LC_ALL=C sort`), so uppercase sorts before lowercase.
- **All names sorted by text**: string and `${…}` dynamic attrs take part
  in the ordering like identifiers, compared by their written text. Nothing
  in an `inherit` statement names a positional order, so no exemption akin
  to `array_sorting`'s is needed.
- **Report**: one per `inherit`, pointed at the first out-of-place name
  (`windows(2)` first-adjacent-out-of-order pattern), message like
  "`{name}` is out of alphabetical order".
- **Rewrite**: `utils::segments` / `utils::rebuild` restricted to the
  positions holding names, keeping the from-expression's position, with
  comments/spacing travelling as the existing rewrites do; fall back to a
  bare diagnostic when the rewrite won't reparse.

## Files to modify

- `lib/src/lints/inherit_ordering.rs` — new lint (docs comment, `#[lint]`,
  `Rule` impl, rewrite fn)
- `lib/src/utils.rs` — possibly a small `inherit_name_positions(items)`-style
  helper or a sort-key helper for inherit attrs
- `lib/src/lints.rs` — register `inherit_ordering` in the `lints!` macro
- `bin/tests/inherit_ordering.rs` — new snapshot tests via `generate_tests!`

## Reuse

- `lib/src/lints/array_sorting.rs` — closest model: `is_multiline` gate,
  `windows(2)` out-of-place detection, `order.sort_by_key`, rebuild +
  reparse for the suggestion.
- `lib/src/lints/attribute_ordering.rs` — byte-order doc rationale,
  report-once shape.
- `lib/src/utils.rs` — `segments`, `rebuild`, `is_multiline`;
  `rnix::ast::Inherit` (`inherit_from()`, `attrs()`) for finding names vs
  the from-expression.

## Steps

- [x] Sort-key / position helper in `utils.rs` for inherit attrs
- [x] New lint + registration (code 39)
- [x] Snapshot tests in `bin/tests/inherit_ordering.rs`
- [x] `cargo insta` accept and eyeball fixed output

## Test cases

- user's example: `inherit (lib)` with `mkOption`/`mkPackageOption` swapped
- already-sorted inherit — clean
- single-line `inherit b a;` and `inherit (lib) mkMerge mkOption;` — sorted
  (no multiline gate)
- multi-name inherit without a from-expression
- each of several inherits in a set sorted independently
- string / `${…}` dynamic attr names sorted by text alongside idents
- comments inside/around names move correctly

## Decisions (confirmed by the user)

1. Single-line inherits are in scope (unlike `array_sorting`).
2. All names — string and dynamic attrs too — are sorted by text; no
   functional-order exemption is needed.
3. Byte order, matching `attribute_ordering`.

## Verification

- `cargo insta test -p statix-tests` in `bin`, review new snapshots
- Manual: paste the user's snippet into a scratch file, run `statix lint`
  and `statix fix` on it

## Implementation notes

- Identify the names via `rnix::ast::Inherit`: `attrs()` gives every
  `Attr` (ident, string, dynamic), `inherit_from()` gives the node to
  anchor — find its `parts.items` index by text range and exclude it from
  the sortable positions.
- Sort key is the attr node's source text, since no quoted/dynamic name
  needs delimiter normalisation (all names sort by what they read like).
- Reparse the rebuilt text and find the top-level `NODE_INHERIT` to build
  the `Suggestion::with_replacement`, as `array_sorting` does with `List`.

## Outcome / deviations

- `inherit_name_positions` matches the parsed kinds (`NODE_IDENT`,
  `NODE_STRING`, `NODE_DYNAMIC`) rather than using `inherit_from()` —
  the from-expression is a `NODE_INHERIT_FROM` child, so the attrs are
  simply every other node kind.
- One deviation discovered while verifying: a bare `inherit` statement is
  not valid Nix at the file root, so the usual reparse-the-rewrite fails
  and the fix would silently degrade to a diagnostic-only report. The
  rewrite is therefore re-parsed wrapped in braces that the suggestion
  never carries — the same spirit as `blank_lines.rs`'s `{text}x` append
  trick.
- Existing `collapsible_inherits` and `inherit_blank_line` snapshots were
  legitimately updated: those test expressions also contain unsorted
  inherit names, now fixed by the new lint when the full suite runs.
