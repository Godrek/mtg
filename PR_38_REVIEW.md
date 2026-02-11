# PR #38 Review: Fix stack overflow at high DEPTH values by converting tail recursion to loops

## Summary

This PR converts tail-recursive calls in `traverse()` and `traverse_goldfish()` to iterative loops, and adds an 8MB rayon thread pool stack size in `commander_goldfish.rs`. The goal is to prevent stack overflows when running with high or unlimited depth (`DEPTH=0`).

## Verdict: Correct, with minor observations

The tail-recursion-to-loop transformation is **semantically correct** in both functions. The PR is ready to merge.

---

## Detailed Analysis

### 1. `traverse()` — Loop conversion (correct)

The function has 5 code paths. Three are tail-recursive and correctly converted to `continue`:

| Path | Original | New | `depth` | `actions_taken` | Correct? |
|------|----------|-----|---------|-----------------|----------|
| Empty actions (forced pass) | Tail recurse, `depth` unchanged | `continue`, depth unchanged | ✅ | ✅ +1 | ✅ |
| Single action (forced) | Tail recurse, `depth` unchanged | `continue`, depth unchanged | ✅ | ✅ +1 | ✅ |
| Opponent node (sampled) | Tail recurse, `depth + 1` | `depth += 1; continue` | ✅ | ✅ +1 | ✅ |

Two paths are non-tail-recursive and correctly remain as recursion:
- **Traverser node**: Explores ALL actions, collects utilities, computes regret — must stay recursive.
- **Terminal/depth-limit paths**: Return directly.

The `mut state` and `mut depth` / `mut actions_taken` parameter changes are correct. `state` was previously consumed by the recursive call; now it's mutated in-place (no clone needed for single-path cases).

**Key detail verified**: The opponent node previously cloned `state` before applying the action, but the clone was unnecessary since the original `state` was never used again. The new code correctly mutates in-place.

### 2. `traverse_goldfish()` — Loop conversion (correct)

Same pattern. Four tail-recursive paths converted to `continue`:

| Path | `depth` change | Correct? |
|------|---------------|----------|
| Opponent (goldfish) action | Unchanged (not a pilot decision) | ✅ |
| Empty actions (pass) | Unchanged | ✅ |
| Single action (forced) | Unchanged | ✅ |

The pilot's traverser node (explore all actions) correctly remains recursive.

**Notable**: `depth` is NOT `mut` in `traverse_goldfish` — this is correct because depth is never incremented in any of the looped paths (it only increments at pilot decision nodes, which are still recursive).

### 3. Rayon stack size (correct, minor note)

The 8MB stack size in `commander_goldfish.rs` is a sensible defense-in-depth measure for the remaining recursive traverser nodes. The recursive depth is now bounded by the number of **multi-action traverser decision points** along any single path (not total states visited), which is dramatically smaller than before.

**Observation**: The `build_global()` call sets the rayon pool for the entire process. This is fine since `commander_goldfish` is the only binary.

### 4. Infinite depth (`max_depth=0`) behavior preserved ✅

The depth-limit check `config.max_depth > 0 && depth >= config.max_depth` is unchanged. When `max_depth == 0`, the first conjunct is false, so depth never triggers a cutoff. Games are bounded only by `max_actions` and natural game termination. The loop conversion prevents stack overflow in this unbounded-depth scenario — which was the core motivation.

### 5. No other entry points are missed

- `traverse()` and `traverse_goldfish()` are only called from within `mccfr.rs`
- All external entry points (`train_goldfish_parallel_with_progress`, `train_goldfish_with_progress`, `run_iteration_with_abstraction`, etc.) call these functions indirectly
- `commander_goldfish.rs` is the only binary; tests use the same library code
- The 2-player `train_parallel` also uses rayon but benefits from the loop conversion without needing a separate stack size configuration (test configs use bounded depth)

### 6. Efficiency improvement (bonus)

The loop conversion eliminates unnecessary `state.clone()` calls in single-path cases:
- Old single-action path: `state.clone()` then pass clone to recursion (original dropped)
- New: mutate `state` in-place directly

This reduces allocation pressure, especially in long sequences of forced passes or single-action states.

---

## No issues found

The transformation is faithful to the original semantics. No behavioral changes, no missed edge cases.
