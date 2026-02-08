# PR Review: `codex/add-deck-importing-module`

**PR**: Add deck import from list files
**Branch**: `codex/add-deck-importing-module`
**Files changed**: 4 (164 lines added)
**Reviewer**: Claude (Opus 4.6)

## Summary

This PR adds a deck importing module that parses text files in the common
`<quantity> <card name>` format (e.g., `4 Lightning Bolt`) and produces a
`Decklist`. It also adds a `find_by_name` method to `CardDatabase` and includes
a basic integration test.

**Build status**: Compiles cleanly. No clippy warnings on new code. Test passes.

---

## What's Good

1. **Well-structured error handling** -- `DeckImportError` is a proper enum with
   `Display`, `Error`, and `From<io::Error>` implementations. Error messages
   include line numbers and the offending line content, which is helpful for
   debugging malformed deck files.

2. **Correct deduplication** -- Duplicate card entries are merged by summing
   quantities via the `indices` HashMap. A file listing `2 Lightning Bolt` on
   one line and `1 Lightning Bolt` later correctly produces a single entry with
   quantity 3.

3. **Edge case handling** -- Empty lines are skipped, zero-quantity entries are
   rejected, whitespace is trimmed, and card lookup is case-insensitive. These
   are all practical decisions for a deck importer.

4. **Clean integration** -- The `find_by_name` addition to `CardDatabase` is
   minimal and placed next to the existing `get()` method. The module is
   registered in `lib.rs` correctly.

---

## Issues

### 1. `find_by_name` is O(n) linear scan [medium]

**File**: `src/game/mod.rs:241-247`

```rust
pub fn find_by_name(&self, name: &str) -> Option<CardId> {
    let target = name.trim();
    self.cards
        .values()
        .find(|card| card.name.eq_ignore_ascii_case(target))
        .map(|card| card.id)
}
```

`CardDatabase.cards` is a `HashMap<CardId, CardDef>`. This iterates over every
card definition on every call. For a 15-line deck file this is negligible, but
if the card database grows to thousands of cards or this method gets used in
hot paths, it becomes a bottleneck.

**Recommendation**: Add a `name_index: HashMap<String, CardId>` (keyed on
lowercased name) to `CardDatabase`, populated on `insert()`. This makes lookups
O(1). At minimum, add a doc comment noting the linear scan cost so future
callers are aware.

### 2. No comment line support [medium]

**File**: `src/deck_import.rs:79-88`

Many deck list formats support comment lines starting with `//` or `#`. A line
like `// My awesome deck` would currently fail with `InvalidLine` ("quantity is
not a number") since it tries to parse `//` as a number.

**Recommendation**: Skip lines starting with `//` or `#` after trimming.

### 3. No sideboard support [low]

Standard MTG deck files often have a sideboard section separated by a blank line
or a `Sideboard:` / `SB:` marker. Currently, blank lines are silently skipped,
so a file with a sideboard section would import all cards (main + side) into one
flat list.

**Recommendation**: Either document that sideboards are not supported, or add
basic parsing to separate main deck from sideboard. Even just stopping at a
`Sideboard:` line would be an improvement.

### 4. Test coverage is thin [medium]

**File**: `tests/deck_import_test.rs`

There is one test covering the happy path. Missing coverage for:

- Invalid quantity (non-numeric, zero)
- Unknown card name (`UnknownCard` error variant)
- Empty file / file with only blank lines
- Missing file (I/O error)
- Deduplication assertion (the test checks `total_cards() == 4` but doesn't
  verify that Lightning Bolt appears as a single entry with quantity 3)

**Recommendation**: Add tests for at least the error paths (`InvalidLine`,
`UnknownCard`) and verify deduplication explicitly.

### 5. Use `split_once` instead of `find` + `split_at` [low / style]

**File**: `src/deck_import.rs:86-96`

```rust
let first_space = trimmed
    .find(|c: char| c.is_whitespace())
    .ok_or_else(|| DeckImportError::InvalidLine { ... })?;
let (qty_str, rest) = trimmed.split_at(first_space);
```

More idiomatic with `split_once`:

```rust
let (qty_str, name) = trimmed
    .split_once(char::is_whitespace)
    .ok_or_else(|| DeckImportError::InvalidLine { ... })?;
let name = name.trim_start();
```

This is cleaner and avoids the manual `split_at` + trim.

### 6. Deck name derived from filename may surprise callers [low]

**File**: `src/deck_import.rs:65-71`

The deck name is derived from the file stem. A path like `/tmp/.txt` fails with
`MissingDeckName`, which is reasonable. But there's no way for a caller to
override the name without modifying the returned `Decklist`. Consider accepting
an optional name parameter, or documenting this behavior.

---

## Verdict

**Approve with suggestions.** The core implementation is solid, correct, and
well-structured. The error handling is thorough and the code integrates cleanly
with the existing codebase. The main feedback centers on test coverage (more
error-path tests needed), the O(n) name lookup (fine for now but worth noting),
and usability improvements (comment lines, `split_once`). None of these are
blockers for merging.
