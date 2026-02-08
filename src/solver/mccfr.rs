//! Phase 1B.3 — External Sampling MCCFR Traversal
//!
//! Implements the external sampling variant of Monte Carlo Counterfactual
//! Regret Minimization (MCCFR). In each iteration, one player is the
//! "traverser" — all their actions are explored — while the opponent's
//! actions and chance outcomes are sampled according to the current strategy.
//!
//! # Algorithm Overview
//!
//! ```text
//! for each iteration:
//!   for each player p:
//!     traverse(root_state, p)
//!
//! traverse(state, traverser):
//!   if terminal(state): return utility(state, traverser)
//!   if chance(state): sample outcome, recurse
//!
//!   player = to_act(state)
//!   actions = legal_actions_abstracted(state)
//!   canonical = actions.map(|a| canonicalize(a, state))
//!   info_set = InformationSet::from_view(state.visible_state(player))
//!   strategy = regret_match(info_set, canonical)
//!
//!   if player == traverser:
//!     // Explore ALL actions
//!     for each action a:
//!       child = apply(state.clone(), a)
//!       util[a] = traverse(child, traverser)
//!     node_util = sum(strategy[a] * util[a])
//!     for each action a:
//!       regret[canonical[a]] += util[a] - node_util
//!     return node_util
//!   else:
//!     // Sample ONE action from opponent's strategy
//!     a = sample(strategy)
//!     child = apply(state.clone(), a)
//!     return traverse(child, traverser)
//! ```
//!
//! # Key Design Decisions
//!
//! - Uses `legal_actions_abstracted()` (bucketed combat) to bound branching factor
//! - Uses `canonicalize()` to map concrete actions to stable identifiers for
//!   regret table keying — actions are identified by card identity, not ObjectId
//! - Skips CFR nodes with only one legal action (trivial pass-through)
//! - Depth-limited: returns heuristic evaluation after `max_depth` plies
//! - Only increments depth at multi-action decision nodes, not forced passes

use rand::Rng;

use crate::action::canonical::canonicalize;
use crate::action::{legal_actions_abstracted, Action};
use crate::game::{GameState, PlayerIndex};
use crate::info_set::InformationSet;
use crate::rules;
use crate::solver::{sample_from_distribution, RegretTable};

/// Configuration for MCCFR training.
#[derive(Debug, Clone)]
pub struct McfrConfig {
    /// Maximum depth (decision points) before falling back to heuristic evaluation.
    /// 0 = unlimited (full game tree).
    pub max_depth: u32,
    /// Maximum actions before declaring a draw (prevents infinite loops).
    pub max_actions: u32,
}

impl Default for McfrConfig {
    fn default() -> Self {
        McfrConfig {
            max_depth: 0, // unlimited
            max_actions: 10_000,
        }
    }
}

/// Run one complete MCCFR iteration: traverse for both players.
///
/// This is the main entry point for the training loop. Each call
/// performs two traversals (one per player), updating regret tables
/// for both players.
pub fn run_iteration(
    initial_state: &GameState,
    regret_tables: &mut [RegretTable; 2],
    config: &McfrConfig,
) {
    for traverser in 0..2 {
        let state = initial_state.clone();
        let mut rng = rand::thread_rng();
        traverse(
            state,
            traverser,
            regret_tables,
            config,
            0,
            0,
            &mut rng,
        );
    }
}

/// Recursive MCCFR traversal.
///
/// Returns the counterfactual utility for the `traverser` at this node.
fn traverse(
    state: GameState,
    traverser: PlayerIndex,
    regret_tables: &mut [RegretTable; 2],
    config: &McfrConfig,
    depth: u32,
    actions_taken: u32,
    rng: &mut impl Rng,
) -> f64 {
    // Terminal check: game over
    if state.game_over {
        return terminal_utility(&state, traverser);
    }

    // Action limit: use heuristic evaluation
    if actions_taken >= config.max_actions {
        return heuristic_utility(&state, traverser);
    }

    let player = state.priority_player;
    let actions = legal_actions_abstracted(&state);

    // No actions available — pass priority (not a decision node)
    if actions.is_empty() {
        let mut next_state = state;
        rules::apply_action(&mut next_state, &Action::PassPriority);
        return traverse(
            next_state,
            traverser,
            regret_tables,
            config,
            depth, // don't increment depth for forced pass
            actions_taken + 1,
            rng,
        );
    }

    // Only one legal action — no decision to make, skip CFR node
    if actions.len() == 1 {
        let mut next_state = state.clone();
        rules::apply_action(&mut next_state, &actions[0]);
        return traverse(
            next_state,
            traverser,
            regret_tables,
            config,
            depth, // don't increment depth for forced action
            actions_taken + 1,
            rng,
        );
    }

    // Depth limit at multi-action decision nodes
    if config.max_depth > 0 && depth >= config.max_depth {
        return heuristic_utility(&state, traverser);
    }

    // Canonicalize all legal actions for stable regret table keying
    let canonical_actions: Vec<_> = actions
        .iter()
        .map(|a| canonicalize(a, &state))
        .collect();

    // Compute information set for the acting player
    let view = state.visible_state(player);
    let info_set = InformationSet::from_view(&view, state.card_db());
    let info_hash = info_set.hash_value();

    // Get or create regret table entry and compute current strategy
    let strategy = {
        let entry = regret_tables[player].get_or_create(info_hash);
        entry.current_strategy(&canonical_actions)
    };

    if player == traverser {
        // Traverser node: explore ALL actions, compute counterfactual regret
        let num_actions = actions.len();
        let mut action_utilities = vec![0.0f64; num_actions];

        for (i, action) in actions.iter().enumerate() {
            let mut child_state = state.clone();
            rules::apply_action(&mut child_state, action);
            action_utilities[i] = traverse(
                child_state,
                traverser,
                regret_tables,
                config,
                depth + 1,
                actions_taken + 1,
                rng,
            );
        }

        // Expected utility under current strategy
        let node_utility: f64 = strategy
            .iter()
            .zip(action_utilities.iter())
            .map(|(&s, &u)| s * u)
            .sum();

        // Update cumulative regret and strategy for each action via canonical key
        let entry = regret_tables[player].get_or_create(info_hash);
        for (i, ca) in canonical_actions.iter().enumerate() {
            let action_entry = entry.get_or_create_action(ca);
            action_entry.cumulative_regret += action_utilities[i] - node_utility;
            action_entry.cumulative_strategy += strategy[i];
        }
        entry.visit_count += 1;

        node_utility
    } else {
        // Opponent node: sample ONE action from their current strategy
        let action_idx = sample_from_distribution(&strategy, rng);
        let action = &actions[action_idx];

        // Update opponent's cumulative strategy via canonical keys
        let entry = regret_tables[player].get_or_create(info_hash);
        for (i, ca) in canonical_actions.iter().enumerate() {
            entry.get_or_create_action(ca).cumulative_strategy += strategy[i];
        }
        entry.visit_count += 1;

        let mut child_state = state.clone();
        rules::apply_action(&mut child_state, action);
        traverse(
            child_state,
            traverser,
            regret_tables,
            config,
            depth + 1,
            actions_taken + 1,
            rng,
        )
    }
}

/// Terminal utility: +1 for win, -1 for loss, 0 for draw.
fn terminal_utility(state: &GameState, player: PlayerIndex) -> f64 {
    match state.winner {
        Some(w) if w == player => 1.0,
        Some(_) => -1.0,
        None => 0.0,
    }
}

/// Heuristic evaluation when depth limit is reached.
///
/// Uses a simple life-total-based evaluation:
/// - Positive when we're ahead on life
/// - Normalized to [-1, 1] range
fn heuristic_utility(state: &GameState, player: PlayerIndex) -> f64 {
    let opp = state.opponent(player);
    let my_life = state.players[player].life as f64;
    let opp_life = state.players[opp].life as f64;

    // Life advantage, normalized by starting life (20)
    let life_diff = my_life - opp_life;
    let normalized = (life_diff / 20.0).clamp(-1.0, 1.0);

    // Board presence bonus
    let db = state.card_db();
    let my_power: i32 = state
        .creatures_controlled_by(player)
        .iter()
        .map(|&id| {
            let inst = &state.objects[&id];
            let def = db.get(inst.card_def_id).unwrap();
            inst.effective_power(def)
        })
        .sum();
    let opp_power: i32 = state
        .creatures_controlled_by(opp)
        .iter()
        .map(|&id| {
            let inst = &state.objects[&id];
            let def = db.get(inst.card_def_id).unwrap();
            inst.effective_power(def)
        })
        .sum();

    let board_diff = (my_power - opp_power) as f64 / 10.0;
    let board_normalized = board_diff.clamp(-0.5, 0.5);

    (normalized * 0.7 + board_normalized * 0.3).clamp(-1.0, 1.0)
}

/// Training loop: run many MCCFR iterations and return the trained regret tables.
///
/// This is the high-level training function for the minimal scenario.
pub fn train(
    initial_state: &GameState,
    num_iterations: u32,
    config: &McfrConfig,
) -> [RegretTable; 2] {
    let mut regret_tables = [RegretTable::new(), RegretTable::new()];

    for _ in 0..num_iterations {
        run_iteration(initial_state, &mut regret_tables, config);
    }

    regret_tables
}

/// Compute a rough measure of exploitability by comparing the two players'
/// expected values. In a perfect Nash equilibrium of a zero-sum game,
/// both players' values sum to zero and neither can improve unilaterally.
///
/// Lower values indicate closer approximation to Nash equilibrium.
/// This is an approximation — true exploitability requires a best-response
/// computation, which is expensive.
pub fn approximate_exploitability(regret_tables: &[RegretTable; 2]) -> f64 {
    // Sum of absolute average regret across all info sets
    let mut total_regret = 0.0;
    let mut total_entries = 0;

    for table in regret_tables {
        for (_, data) in &table.data {
            if data.visit_count == 0 {
                continue;
            }
            let avg_regret: f64 = data
                .action_data
                .values()
                .map(|e| e.cumulative_regret.max(0.0))
                .sum::<f64>()
                / data.visit_count as f64;
            total_regret += avg_regret;
            total_entries += 1;
        }
    }

    if total_entries > 0 {
        total_regret / total_entries as f64
    } else {
        f64::INFINITY
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_terminal_utility() {
        let mut state = GameState::new(2);
        state.game_over = true;
        state.winner = Some(0);
        assert_eq!(terminal_utility(&state, 0), 1.0);
        assert_eq!(terminal_utility(&state, 1), -1.0);

        state.winner = None;
        assert_eq!(terminal_utility(&state, 0), 0.0);
    }

    #[test]
    fn test_heuristic_utility_equal_life() {
        use crate::card::sample;
        use std::sync::Arc;

        let db = sample::build_sample_db();
        let mut state = GameState::new(2);
        state.card_db = Some(Arc::new(db));

        // Equal life totals, no creatures — should be close to 0
        let util = heuristic_utility(&state, 0);
        assert!(util.abs() < 0.01, "Equal game should have ~0 utility, got {}", util);
    }

    #[test]
    fn test_heuristic_utility_life_advantage() {
        use crate::card::sample;
        use std::sync::Arc;

        let db = sample::build_sample_db();
        let mut state = GameState::new(2);
        state.card_db = Some(Arc::new(db));
        state.players[0].life = 20;
        state.players[1].life = 10;

        let util = heuristic_utility(&state, 0);
        assert!(util > 0.0, "Player with more life should have positive utility");
    }
}
