# PR #11 Review: Combat Action Abstraction for MCCFR

## Summary

This PR introduces a `CombatAbstraction` system that buckets attacker/blocker
combinations into a small number of strategically distinct postures, reducing
the action space from O(2^n) to O(1) for MCCFR traversal. The approach is
sound and well-motivated.

**Verdict: Approve with suggestions**

All 24 tests pass. The code compiles cleanly. The design is reasonable for an
MCCFR action abstraction layer. Below are findings organized by severity.

---

## Issues

### 1. `legal_actions_with` is not threaded through `DeclareBlockers` from `legal_actions` (Medium)

**File:** `src/action/mod.rs`

The `legal_actions_with` function correctly passes the `abstraction` parameter
to the `DeclareAttackers` and `DeclareBlockers` branches. However, there is an
implicit coupling: when the game reaches `DeclareBlockers`, it depends on which
attacker subsets were generated during `DeclareAttackers`. If MCCFR uses
`legal_actions_abstracted` for attackers but the engine resolves the game using
`legal_actions` (full) for blockers (or vice versa), the blocking response
won't be calibrated to the abstracted attacker set. This is fine as long as the
caller consistently uses one or the other, but it should be documented.

### 2. Blocker bucket "chump-biggest" only assigns one blocker (Low-Medium)

**File:** `src/action/mod.rs`, `generate_block_buckets`

The "chump-biggest" bucket breaks after assigning a single blocker to a single
attacker (`if assignment.len() >= 1 { break; }`). This is by design per the
doc comment, but in practice "chump-blocking" often involves chumping multiple
attackers simultaneously. A variant that chumps all unblockable-but-lethal
attackers would capture a common strategic posture that the current buckets miss.

### 3. Favorable-only uses smallest blocker first, may miss better matchups (Low)

**File:** `src/action/mod.rs`, `generate_block_buckets`, bucket 3

The favorable-only bucket iterates `blocker_by_power_asc` (smallest first) for
each attacker (biggest first). This greedy approach may assign a small blocker
to a big attacker where it both survives and kills, consuming that blocker
before it could have been used on an even better matchup. Since the greedy
assignment order affects which favorable blocks get found, some favorable
matchups may be missed. This is acceptable for an abstraction but worth noting.

### 4. `effective_power` returns `i32` but damage comparison assumes non-negative (Low)

**File:** `src/action/mod.rs`, `generate_block_buckets`

The `effective_power` and `effective_toughness` methods return `i32`, which
can be negative (e.g., a creature with -1/-1 counters exceeding its base).
The blocking bucket comparisons like `blocker_stats[bi].power >=
attacker_stats[ai].toughness` work correctly with negative values in Rust's
signed arithmetic, but a creature with 0 or negative power would "kill"
an attacker with 0 or negative toughness. This is a corner case that's
unlikely in practice but worth being aware of.

### 5. `Vigilance` not considered in evasion bucket categorization (Low)

**File:** `src/action/mod.rs`, `generate_attack_buckets`

The evasion bucket includes Flying, Fear, Intimidate, and Menace. Vigilance is
not an evasion keyword, so its omission is correct. However, vigilant creatures
are strategically distinct because they attack without tapping (preserving
blocking ability). A "vigilance-only" or "safe attackers" bucket could be
valuable for MCCFR since attacking with vigilant creatures is strictly lower
risk. This is a potential future enhancement, not a bug.

### 6. Missing `Unblockable` / protection-based evasion (Low)

The evasion bucket checks for Flying, Fear, Intimidate, and Menace. If future
cards have protection or other forms of evasion (shadow, skulk, etc.), the
evasion bucket won't capture them. The current keyword set covers the existing
card pool, so this is fine for now.

---

## Design Observations

### Strengths

- **Clean API separation**: `legal_actions()` unchanged, `legal_actions_abstracted()`
  is the new entry point. Existing callers are unaffected.
- **Fallback threshold**: The `eligible.len() > 5` threshold (32 subsets) for
  switching to buckets is well-chosen; it preserves exact play on small boards.
- **Deduplication**: Using `HashSet` to deduplicate buckets prevents redundant
  actions when different bucket heuristics produce identical subsets.
- **Test coverage**: 7 new tests covering small-board fallback, large-board
  reduction, bucket invariants (none/alpha always present), evasion bucket,
  dedup, blocking reduction, and full-game completion.

### Test quality

The tests are well-structured with clear assertions. The `setup_combat_state`
helper is clean and reusable. The `test_abstracted_game_completes` test is
particularly valuable as a smoke test that the abstraction doesn't break game
flow. One suggestion: add a test that verifies the `best-1` bucket actually
picks the highest-power creature (not just that a size-1 bucket exists).

---

## Minor Style Notes

- The `add_bucket` / `add_assignment` closures take `&mut HashSet` and
  `&mut Vec` parameters. These could be methods on a small helper struct to
  reduce parameter passing noise, but the current approach works.
- The `CreatureStats` struct is defined inside `generate_block_buckets`. This
  is fine for locality but could be shared with `generate_attack_buckets` if
  both needed the same stats.

---

## Conclusion

The PR achieves its goal of reducing combat action space from exponential to
constant for MCCFR. The implementation is correct, well-tested, and
non-breaking. The main suggestions are around expanding bucket coverage
(multi-chump, vigilance-aware attacks) and documenting the consistency
requirement between attacker and blocker abstraction levels.
