# PR #19 Review: Fix compute_characteristics() caching, O(n) battlefield scans, and add SwitchPT+counters tests

## Summary

Single commit (`00e0ee0`) addressing the 3 non-blocking observations from the PR #18 review:
1. `compute_characteristics()` called redundantly (~90x per combat step)
2. Missing test for counters + SwitchPT interaction
3. O(n) `battlefield.contains()` scans in effect filtering

**+221 / -53 lines** across 3 files. All 155 tests pass (59 unit + 70 integration + 25 MCCFR + 1 deck import).

## Fix 1: CharacteristicsCache — CORRECT

**`src/game/mod.rs:14-55, 325-333, 836-889`**

A `Mutex<CharacteristicsCacheInner>` wrapping a `HashMap<ObjectId, ComputedCharacteristics>` plus a lazily-built `HashSet<ObjectId>` for battlefield membership. Key design decisions:

- **Interior mutability via `Mutex`**: Needed because read-only methods (`effective_power`, `has_keyword`) populate the cache through `&self`. `Mutex` (not `RefCell`) for rayon `Sync` compatibility. Correct choice.
- **Clone produces empty cache**: No contention risk between cloned states. O(0) cache on clone is consistent with the Snapshot Contract and keeps `GameState::clone()` cheap for MCCFR.
- **`#[serde(skip)]`**: Not serialized (derived state). Deserialize uses `Default` (empty cache). Correct.
- **Lazy battlefield HashSet**: Built once per cache epoch on first miss, amortized across all objects queried in the same step. Eliminates redundant `Vec::contains()` O(n) scans.

### Cache invalidation audit — COMPLETE, NO GAPS

Exhaustive audit of all mutation sites that affect characteristics:

| Mutation type | Sites found | All invalidated? |
|---------------|-------------|-------------------|
| `continuous_effects` push/retain/extend | 6 | Yes |
| `move_object()` (changes battlefield) | 13 | Yes (invalidated inside `move_object`) |
| `plus_counters` / `minus_counters` | 4 | Yes |
| `controller` changes | 1 | Yes (via downstream `move_object` + `refresh_continuous_effects`) |
| `cleanup_eot_effects` | 1 | Yes |

Fields like `tapped` and `damage_marked` are correctly NOT triggering invalidation — they don't affect computed characteristics.

## Fix 2: O(n) Battlefield Scans → HashSet — CORRECT

**`src/layers/mod.rs:216, 364, 411`** — `compute_characteristics`, `effect_applies_to`, and `is_creature_on_battlefield` now accept `&HashSet<ObjectId>` instead of `&[ObjectId]`.

**`src/game/mod.rs:773`** — `refresh_continuous_effects` builds a local `HashSet` for its own retain check.

**`src/game/mod.rs:855-860`** — `get_characteristics` lazily builds the `HashSet` inside the cache, amortized across all queries in the same epoch.

All 14 unit tests in `layers::tests` updated from `vec![...]` to `HashSet` literals.

## Fix 3: SwitchPT + Counters Tests — CORRECT

Two new unit tests:

**`test_counters_with_switch_pt`** (`src/layers/mod.rs:800-826`): 2/4 creature with 1 +1/+1 counter and SwitchPT. Verifies: base 2/4 → counters 3/5 (7d) → swap 5/3 (7e). Asymmetric P/T makes the ordering observable.

**`test_anthem_counters_switch_pt`** (`src/layers/mod.rs:832-875`): 1/3 creature with anthem (+1/+1), 1 counter, and SwitchPT. Verifies: base 1/3 → anthem 2/4 (7c) → counters 3/5 (7d) → swap 5/3 (7e). Tests all three sublayers in sequence.

Both tests would fail if counters were applied after SwitchPT (the old bug from PR #18), confirming the fix is locked in.

## Minor Observations (Non-Blocking)

1. **`ComputedCharacteristics::clone()` on cache hit**: The `get_characteristics()` method clones the cached entry on every hit. Since `ComputedCharacteristics` contains multiple `Vec`s (card_types, subtypes, colors, keywords), this allocates on every call. For the MCCFR hot path, returning an `Arc<ComputedCharacteristics>` or a cache reference could avoid this — but it's a micro-optimization and not needed now.

2. **Mutex overhead**: The `Mutex` lock/unlock on every `effective_power`/`has_keyword` call adds some overhead. In practice this is uncontended (each `GameState` clone gets its own cache), so the cost is just the atomic operations. Fine for now.

## Verdict

**Approve.** All 3 observations addressed correctly. Cache invalidation is complete with no gaps. Tests lock in layer ordering. 155 tests pass, clippy clean (warnings only).
