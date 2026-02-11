# PR #44 Review: Add MCTS solver for goldfish solitaire optimization

## Overview

This PR adds a Monte Carlo Tree Search (MCTS) solver for goldfish (solitaire) optimization. It introduces 3 new files and modifies 2 existing ones, totaling +1,244 lines. The code compiles cleanly and all 5 unit tests pass.

**Commit:** `cc91833` — Add MCTS solver for goldfish solitaire optimization

**Files changed:**
- `src/solver/mcts.rs` (new, 761 lines) — Core MCTS tree, UCB1, search, MctsStrategy
- `src/bin/mcts_goldfish.rs` (new, 314 lines) — CLI binary with Greedy baseline comparison
- `src/simulation/mod.rs` (+168 lines) — Simulation wrappers and parallel aggregation
- `src/solver/mod.rs` (+1 line) — Module re-export

## Architecture & Design

- Using MCTS for single-agent solitaire optimization is the correct choice vs MCCFR (which targets Nash equilibria for adversarial games). Good framing in the module docs.
- The `MctsStrategy` implementing `Strategy` trait allows plug-and-play use with the existing simulation infrastructure.
- Clean separation: core tree search in `mcts.rs`, simulation wrappers in `simulation/mod.rs`, CLI in `mcts_goldfish.rs`.
- The greedy-baseline-then-MCTS-comparison pattern in the CLI binary is useful for validating that MCTS actually improves over greedy play.
- The reward function `(MAX_TURN + 1 - T) / MAX_TURN` correctly incentivizes faster kills with values in (0, 1].
- Using most-visited child for final action selection (rather than highest Q-value) is the standard robust approach.

## Issues to Address

### 1. Double-counting visits in `tree_walk` (bug)

**File:** `src/solver/mcts.rs`, `tree_walk()` function

When a node is expanded for the first time, the reward is backpropagated into the newly expanded node's `visits` and `total_reward`:

```rust
// First visit to a new node: rollout from here
let reward = rollout(state, rollout_strategy, goldfish_strategy, config);
node.visits += 1;          // <-- incremented here
node.total_reward += reward; // <-- incremented here
return reward;
```

Then the **caller** (either `mcts_search` at the root, or the recursive parent `tree_walk`) also increments the child's stats:

```rust
// Backpropagate
children[child_idx].node.visits += 1;          // <-- incremented again
children[child_idx].node.total_reward += reward; // <-- incremented again
```

This means the first visit to any newly-expanded node gets double-counted (visits=2, total_reward=2*reward after one actual rollout).

**Fix:** Remove `node.visits += 1; node.total_reward += reward;` from the expansion branch in `tree_walk`, since the parent's backpropagation already handles it.

### 2. Root node stats also double-counted in `mcts_search`

Same pattern at the root level: `mcts_search()` increments `root.visits` and `root.total_reward` after each iteration, but if the root is a leaf that gets expanded on iteration 0, it also gets counted inside `tree_walk`'s expansion branch.

### 3. `reuse_tree` config option declared but never used

`MctsConfig::reuse_tree` is defined and documented but never checked anywhere. `MctsStrategy::choose_action()` creates a fresh `MctsNode::new()` every call. Either implement tree reuse or remove the field to avoid confusion.

### 4. Potential stack overflow from unbounded recursion in `tree_walk`

`tree_walk` recurses through forced-pass actions (empty/single PassPriority) and goldfish (player 1) actions without incrementing `depth`. In a game with many consecutive forced passes or goldfish turns, this could build up deep call stack frames. The CLI binary mitigates this with an 8MB rayon stack, but `MctsStrategy::choose_action()` runs on whatever stack the caller provides.

Given the project's history with stack overflows (PR #38 fixed the same issue in MCCFR by converting tail recursion to loops), consider converting the forced-pass/goldfish loops to iteration.

### 5. `check_state_based_actions` called only every 10 actions

In `rollout()` and `run_mcts_goldfish_game()`, SBAs are only checked every 10 actions. The existing `run_goldfish_game()` in the simulation module does the same, so this is **consistent with existing code** — but it means up to 9 actions can occur in a potentially invalid state. Noting for awareness; not a regression.

## Suggestions (non-blocking)

### 6. UCB1 tie-breaking is deterministic

When multiple children have the same UCB1 score (common when all children are unvisited — they all get `INFINITY`), the first one is always selected. This biases exploration toward actions earlier in the legal-actions list. Random tie-breaking or shuffling unvisited children would improve search diversity.

### 7. `format_action` duplicates display logic

The `format_action()` helper reimplements action-to-string formatting that likely overlaps with `Action`'s `Display` impl or similar formatting in the interactive binary. Consider reusing existing display infrastructure.

### 8. Atomic reward aggregation loses precision

`aggregate_mcts_goldfish_results()` uses `total_reward_x1000` (`AtomicU64` scaled by 1000) to accumulate floating-point averages. This truncates to 3 decimal places per game. For large game counts this is fine, but collecting into a `Vec` and aggregating after (or using a `Mutex<f64>`) would preserve full precision with negligible performance impact.

### 9. No integration tests

The PR adds 5 unit tests for MCTS primitives (reward, UCB1, node operations), which is good. Consider adding at least one integration test that runs a small number of MCTS goldfish games (e.g., 2-5 games with low iterations) to verify end-to-end correctness and catch regressions.

## Summary

The core MCTS algorithm is well-implemented and follows established codebase patterns. The **double-counting bug (#1/#2) should be fixed before merge** as it affects search quality (inflated visit counts and skewed Q-values). The unused `reuse_tree` field (#3) and recursion depth concern (#4) are worth addressing. The remaining items are suggestions for future improvement.
