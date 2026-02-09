# PR #26 Review (Round 2): MCCFR goldfish training for solitaire strategy optimization

## Summary

This PR adds goldfish (solitaire) MCCFR training — a single-agent optimization where the
solver trains a pilot player's policy against a fixed passive opponent (`GoldfishStrategy`).
The opponent's action space collapses to deterministic choices, which converges faster
than standard 2-player MCCFR.

**Files changed:** 3 files
- `src/solver/mccfr.rs` — `train_goldfish`, `train_goldfish_with_abstraction`, `traverse_goldfish`
- `src/main.rs` — MCCFR goldfish demo with `goldfish_mccfr_report` helper
- `tests/mccfr_test.rs` — 5 new tests

**All 235 tests pass with zero regressions.**

---

## Round 1 issues — all resolved

| # | Issue | Status |
|---|-------|--------|
| 1 | Unused `rng` parameter in `traverse_goldfish` | **Fixed** — removed entirely |
| 2 | No rollout support at depth limit | **Fixed** — doc comment now explains rationale |
| 3 | Test comment says "20 iters", code uses 10 | **Fixed** — now says "10 iters" |
| 4 | Hardcoded player indices | **Fixed** — `pilot: PlayerIndex` parameter added |
| 5 | `main.rs` duplication | **Fixed** — extracted `goldfish_mccfr_report` helper |

---

## Round 2 assessment

The fix commit (`c4a9345`) cleanly addresses all five review items:

- **`rng` removal**: Cleanly stripped from `traverse_goldfish` signature and all call sites.
  No dead code remains.

- **`pilot` parameter**: `traverse_goldfish` and `train_goldfish_with_abstraction` now accept
  a `pilot: PlayerIndex`. The opponent check changed from `player == 1` to `player != pilot`,
  and all `terminal_utility`/`heuristic_utility`/`visible_state` calls use `pilot` instead of
  hardcoded `0`. `train_goldfish` defaults to `pilot=0` for backward compatibility.

- **Rollout rationale**: The `traverse_goldfish` doc comment now explicitly explains why
  rollouts are omitted and how they could be added if needed.

- **`main.rs` helper**: The new `goldfish_mccfr_report` function eliminates the duplication
  cleanly. Good function signature with all the right parameters.

- **Test fix**: Trivially correct.

No new issues introduced. The code is clean, well-documented, and all tests pass.

---

## Verdict

**Approved.** All review items addressed. Ready to merge.
