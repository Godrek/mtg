# Review: PR #9 — Wrap CardDatabase in Arc for O(1) GameState cloning

## Summary

This PR changes `GameState.card_db` from `Option<CardDatabase>` to
`Option<Arc<CardDatabase>>` so that `GameState::clone()` avoids deep-copying
the entire card database. The card database is immutable after setup, so `Arc`
is the correct sharing primitive. The change is minimal (3 files, +10/−6 lines)
and well-motivated by the commit message.

**Verdict: Approve with required fixes** — the optimization is sound, but there
is a merge-time compilation failure and a missed optimization opportunity in the
simulation hot path.

---

## Blocking Issues

### 1. Post-merge compilation failure in tests

The PR was branched before PR #8 landed. PR #8 added three new test functions
(`test_cleanup_requires_discard_action`, `test_cleanup_allows_pass_at_seven`,
`test_cleanup_multiple_discards`) that assign `state.card_db = Some(db)` without
`Arc::new()`. Git merges the files cleanly (no textual conflict), but the result
fails to compile:

```
error[E0308]: mismatched types
   --> tests/integration_test.rs:229:26
    |
229 |     state.card_db = Some(db);
    |                          ^^ expected `Arc<CardDatabase>`, found `CardDatabase`
```

Same error on lines 257 and 276.

**Fix:** Rebase onto mainline and wrap the three remaining assignment sites with
`Arc::new()`.

---

## Suggestions

### 2. Avoid redundant full clone per game in `simulate()`

In `src/simulation/mod.rs:97`:

```rust
state.card_db = Some(Arc::new(card_db.clone()));
```

This deep-clones `CardDatabase` and then wraps the copy in an `Arc` — once per
game. In the `simulate()` function, N parallel games each do a full DB clone.
The whole point of `Arc` is to avoid this cost.

Consider creating the `Arc` once in the caller and threading it through:

```rust
// In simulate() or at the call site:
let shared_db = Arc::new(card_db.clone());  // one clone total

// In run_game_inner():
state.card_db = Some(Arc::clone(&shared_db));  // just an atomic increment
```

This would require changing the function signatures from `&CardDatabase` to
`Arc<CardDatabase>`, but it eliminates the dominant allocation the PR is trying
to fix.

### 3. Documentation out of date

`AGENTS.md:43` still says:

> The `GameState` owns both the `CardDatabase` (via `card_db: Option<CardDatabase>`)

This should be updated to reflect the new `Option<Arc<CardDatabase>>` type. The
borrow-checker guidance in that section still applies (Deref coercion means
`state.card_db()` returns `&CardDatabase` as before), so only the type
annotation in the prose needs updating.

---

## What looks good

- **Correct use of `Arc`**: The card database is immutable after construction,
  so shared ownership via `Arc` is appropriate. No `Mutex` needed.
- **Deref coercion preserves the API**: The `card_db()` accessor still returns
  `&CardDatabase`, so all downstream code (rules engine, action enumeration,
  strategies) works unchanged.
- **`serde(skip)` is compatible**: `Arc` doesn't implement `Serialize`/
  `Deserialize`, but the field was already `#[serde(skip)]`, so this is fine.
- **Clear commit message**: The commit message explains the motivation and the
  specific allocation cost being eliminated.
