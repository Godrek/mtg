use rand::seq::SliceRandom;
use std::sync::atomic::{AtomicU64, Ordering};

use crate::action::canonical::canonicalize;
use crate::action::{legal_actions, legal_actions_abstracted, Action};
use crate::card::ObjectId;
use crate::game::{GameState, PlayerIndex};
use crate::info_set::InformationSet;
use crate::solver::{sample_from_distribution, RegretTable};

/// A strategy decides what action to take given a game state.
/// This is the interface the GTO solver will optimize over.
pub trait Strategy: Send + Sync {
    fn choose_action(&self, state: &GameState, player: PlayerIndex) -> Action;
    fn name(&self) -> &str;
}

/// Random strategy: picks uniformly at random from legal actions.
/// Useful as a baseline and for Monte Carlo rollouts.
pub struct RandomStrategy;

impl Strategy for RandomStrategy {
    fn choose_action(&self, state: &GameState, _player: PlayerIndex) -> Action {
        let mut rng = rand::thread_rng();
        let actions = legal_actions(state);
        actions
            .choose(&mut rng)
            .cloned()
            .unwrap_or(Action::PassPriority)
    }

    fn name(&self) -> &str {
        "Random"
    }
}

/// Greedy heuristic strategy: plays lands, casts biggest spell possible,
/// attacks with everything, blocks favorably.
/// Better than random, serves as a reasonable default opponent.
pub struct GreedyStrategy;

impl Strategy for GreedyStrategy {
    fn choose_action(&self, state: &GameState, _player: PlayerIndex) -> Action {
        let actions = legal_actions(state);
        let db = state.card_db();

        // Mulligan heuristic: keep if hand has 2-5 lands, otherwise mulligan
        if state.phase == crate::game::Phase::Mulligan {
            let player = state.priority_player;
            let ps = &state.players[player];
            if !ps.mulligan_decided {
                let land_count = ps.hand.iter().filter(|&&obj_id| {
                    let inst = &state.objects[&obj_id];
                    db.get(inst.card_def_id).map_or(false, |d| d.is_land())
                }).count();
                // Keep if 2-5 lands, or if already mulliganed twice
                if (2..=5).contains(&land_count) || ps.mulligan_count >= 2 {
                    return Action::MulliganKeep;
                }
                return Action::MulliganMulligan;
            }
            // Bottoming: put the highest-CMC non-land card on bottom
            let mut worst_card = None;
            let mut worst_score = -1i32;
            for action in &actions {
                if let Action::MulliganBottomCard { object_id } = action {
                    let inst = &state.objects[object_id];
                    let def = db.get(inst.card_def_id);
                    let score = if def.map_or(false, |d| d.is_land()) {
                        // Lands get low score (keep them)
                        0
                    } else {
                        // Non-lands scored by CMC (bottom expensive ones)
                        def.map_or(5, |d| d.cmc() as i32)
                    };
                    if score > worst_score {
                        worst_score = score;
                        worst_card = Some(action.clone());
                    }
                }
            }
            return worst_card.unwrap_or(actions[0].clone());
        }

        // Priority 0: If we must order triggers or replacement effects, pick
        // the first ordering (FIFO). A real MCCFR solver would evaluate all
        // orderings; greedy just uses FIFO.
        for action in &actions {
            if matches!(action, Action::OrderTriggers { .. } | Action::ChooseReplacementOrder { .. }) {
                return action.clone();
            }
        }

        // Priority 1: Play a land if we can
        for action in &actions {
            if let Action::PlayLand { .. } = action {
                return action.clone();
            }
        }

        // Priority 2: Cast the most expensive spell we can afford
        // (includes casting commander from command zone)
        let mut best_spell: Option<&Action> = None;
        let mut best_cmc = 0;
        for action in &actions {
            let obj_id = match action {
                Action::CastSpell { object_id, .. } => Some(object_id),
                Action::CastCommander { object_id, .. } => Some(object_id),
                _ => None,
            };
            if let Some(object_id) = obj_id {
                let inst = &state.objects[object_id];
                if let Some(def) = db.get(inst.card_def_id) {
                    let cmc = def.cmc();
                    if cmc >= best_cmc {
                        best_cmc = cmc;
                        best_spell = Some(action);
                    }
                }
            }
        }
        if let Some(spell) = best_spell {
            return spell.clone();
        }

        // Priority 3: Attack with all eligible creatures
        for action in &actions {
            if let Action::DeclareAttackers { attackers } = action {
                if !attackers.is_empty() {
                    // Find the action that attacks with the most creatures
                    let mut best_attack: Option<&Action> = None;
                    let mut max_attackers = 0;
                    for a in &actions {
                        if let Action::DeclareAttackers { attackers } = a {
                            if attackers.len() > max_attackers {
                                max_attackers = attackers.len();
                                best_attack = Some(a);
                            }
                        }
                    }
                    if let Some(attack) = best_attack {
                        return attack.clone();
                    }
                }
            }
        }

        // Priority 4: Block with favorable trades
        for action in &actions {
            if let Action::DeclareBlockers { blocks } = action {
                if !blocks.is_empty() {
                    // Simple heuristic: block if our creature's toughness > attacker's power
                    // (i.e., we survive the block)
                    let mut best_block: Option<&Action> = None;
                    let mut best_score: i32 = 0;

                    for a in &actions {
                        if let Action::DeclareBlockers { blocks } = a {
                            let score = evaluate_blocks(state, blocks);
                            if score > best_score || best_block.is_none() {
                                best_score = score;
                                best_block = Some(a);
                            }
                        }
                    }
                    if best_score > 0 {
                        if let Some(block) = best_block {
                            return block.clone();
                        }
                    }
                }
            }
        }

        // Priority 5: Cleanup discard if forced
        for action in &actions {
            if let Action::Discard { .. } = action {
                return action.clone();
            }
        }

        // Default: pass priority
        Action::PassPriority
    }

    fn name(&self) -> &str {
        "Greedy"
    }
}

/// MCCFR-trained strategy: selects actions according to the average
/// strategy computed by the MCCFR solver.
///
/// After training, the average strategy (not the current strategy) converges
/// to a Nash equilibrium in two-player zero-sum games. This implementation
/// looks up the current info set in the regret table, canonicalizes the
/// available actions, and samples from the average strategy distribution.
///
/// Falls back to uniform random among legal actions when the info set
/// hasn't been visited during training.
pub struct McfrStrategy {
    /// Trained regret table (read-only during play).
    policy: RegretTable,
}

impl McfrStrategy {
    /// Create a new McfrStrategy from a trained regret table.
    pub fn new(policy: RegretTable) -> Self {
        McfrStrategy { policy }
    }
}

impl Strategy for McfrStrategy {
    fn choose_action(&self, state: &GameState, player: PlayerIndex) -> Action {
        let actions = legal_actions_abstracted(state);
        if actions.is_empty() {
            return Action::PassPriority;
        }
        if actions.len() == 1 {
            return actions[0].clone();
        }

        // Canonicalize actions for stable regret table lookup
        let canonical_actions: Vec<_> = actions
            .iter()
            .map(|a| canonicalize(a, state))
            .collect();

        let view = state.visible_state(player);
        let info_set = InformationSet::from_view(&view, state.card_db());
        let info_hash = info_set.hash_value();

        let distribution = match self.policy.get(info_hash) {
            Some(data) => data.average_strategy(&canonical_actions),
            None => {
                // Unseen info set — fall back to uniform random
                let n = actions.len();
                vec![1.0 / n as f64; n]
            }
        };

        let mut rng = rand::thread_rng();
        let idx = sample_from_distribution(&distribution, &mut rng);
        actions[idx].clone()
    }

    fn name(&self) -> &str {
        "MCCFR"
    }
}

/// MCCFR-trained strategy with information set abstraction (Phase 2B).
///
/// Like `McfrStrategy`, but uses an `InfoSetAbstraction` to hash the info set
/// the same way the training did. This is essential when training uses
/// `BucketedAbstraction` — the play-time strategy must use the same abstraction
/// or it will never find matching entries in the regret table.
///
/// When `fallback` is set, untrained states use the fallback strategy instead
/// of uniform random. This is critical for goldfish mode where MCCFR training
/// only covers a fraction of the state space — without a fallback, untrained
/// mid/late game states get uniform random play (50% PassPriority on every
/// decision), which makes the strategy dramatically worse than even Random.
pub struct AbstractedMcfrStrategy {
    /// Trained regret table (read-only during play).
    policy: RegretTable,
    /// Abstraction used during training (must match).
    abstraction: Box<dyn crate::info_set::InfoSetAbstraction>,
    /// Optional fallback strategy for untrained info sets.
    fallback: Option<Box<dyn Strategy>>,
    /// Minimum visit count to trust the trained policy.
    /// Info sets with fewer visits use the fallback instead.
    min_visits: u64,
    /// Count of decisions where the trained policy was used.
    policy_hits: AtomicU64,
    /// Count of decisions where the fallback was used (info set not trained).
    fallback_hits: AtomicU64,
}

impl AbstractedMcfrStrategy {
    /// Create a new strategy from a trained regret table and matching abstraction.
    pub fn new(
        policy: RegretTable,
        abstraction: Box<dyn crate::info_set::InfoSetAbstraction>,
    ) -> Self {
        AbstractedMcfrStrategy {
            policy,
            abstraction,
            fallback: None,
            min_visits: 0,
            policy_hits: AtomicU64::new(0),
            fallback_hits: AtomicU64::new(0),
        }
    }

    /// Create a new strategy with a fallback for untrained states.
    pub fn with_fallback(
        policy: RegretTable,
        abstraction: Box<dyn crate::info_set::InfoSetAbstraction>,
        fallback: Box<dyn Strategy>,
    ) -> Self {
        AbstractedMcfrStrategy {
            policy,
            abstraction,
            fallback: Some(fallback),
            min_visits: 0,
            policy_hits: AtomicU64::new(0),
            fallback_hits: AtomicU64::new(0),
        }
    }

    /// Set the minimum visit count threshold.
    /// Info sets with fewer visits than this will use the fallback strategy.
    pub fn set_min_visits(&mut self, min_visits: u64) {
        self.min_visits = min_visits;
    }

    /// Number of decisions where the trained policy table was used.
    pub fn policy_hit_count(&self) -> u64 {
        self.policy_hits.load(Ordering::Relaxed)
    }

    /// Number of decisions where the fallback strategy was used.
    pub fn fallback_hit_count(&self) -> u64 {
        self.fallback_hits.load(Ordering::Relaxed)
    }

    /// Reset the hit/miss counters to zero.
    pub fn reset_counters(&self) {
        self.policy_hits.store(0, Ordering::Relaxed);
        self.fallback_hits.store(0, Ordering::Relaxed);
    }
}

impl Strategy for AbstractedMcfrStrategy {
    fn choose_action(&self, state: &GameState, player: PlayerIndex) -> Action {
        let actions = legal_actions_abstracted(state);
        if actions.is_empty() {
            return Action::PassPriority;
        }
        if actions.len() == 1 {
            return actions[0].clone();
        }

        let canonical_actions: Vec<_> = actions
            .iter()
            .map(|a| canonicalize(a, state))
            .collect();

        let view = state.visible_state(player);
        let info_set = InformationSet::from_view(&view, state.card_db());
        let info_hash = self.abstraction.abstract_info_set(&info_set);

        let use_policy = match self.policy.get(info_hash) {
            Some(data) if data.visit_count >= self.min_visits => Some(data),
            _ => None,
        };

        match use_policy {
            Some(data) => {
                self.policy_hits.fetch_add(1, Ordering::Relaxed);
                let distribution = data.average_strategy(&canonical_actions);
                let mut rng = rand::thread_rng();
                let idx = sample_from_distribution(&distribution, &mut rng);
                actions[idx].clone()
            }
            None => {
                self.fallback_hits.fetch_add(1, Ordering::Relaxed);
                // Untrained or under-trained state: use fallback or uniform random
                if let Some(ref fb) = self.fallback {
                    fb.choose_action(state, player)
                } else {
                    let n = actions.len();
                    let distribution = vec![1.0 / n as f64; n];
                    let mut rng = rand::thread_rng();
                    let idx = sample_from_distribution(&distribution, &mut rng);
                    actions[idx].clone()
                }
            }
        }
    }

    fn name(&self) -> &str {
        "MCCFR-Abstracted"
    }
}

/// Goldfish strategy: simulates a passive opponent who takes no actions.
///
/// In MTG, "goldfishing" means playing solitaire against an opponent who does
/// nothing — no blocking, no attacking, no spells. This measures the fastest
/// possible clock (kill turn) for a deck. The goldfish:
///
/// - Always passes priority (never casts spells or activates abilities)
/// - Never attacks
/// - Never blocks
/// - Handles mandatory actions minimally (trigger ordering, forced discard)
///
/// Because the goldfish makes no meaningful decisions, simulations run faster
/// and have near-zero branching on the opponent's side, allowing convergence
/// to the deck's theoretical best-case win speed.
pub struct GoldfishStrategy;

impl Strategy for GoldfishStrategy {
    fn choose_action(&self, state: &GameState, player: PlayerIndex) -> Action {
        let actions = legal_actions(state);

        // Mulligan: always keep; bottom first card if forced (shouldn't happen
        // since keeping at mulligan_count=0 means no bottoming, but handle it
        // defensively so GoldfishStrategy is safe to use in any context).
        if state.phase == crate::game::Phase::Mulligan {
            for action in &actions {
                if matches!(action, Action::MulliganBottomCard { .. }) {
                    return action.clone();
                }
            }
            return Action::MulliganKeep;
        }

        // Handle mandatory actions that can't be skipped

        // Trigger ordering, replacement ordering, and damage assignment:
        // pick the first option offered (FIFO / default ordering).
        for action in &actions {
            if matches!(
                action,
                Action::OrderTriggers { .. }
                    | Action::ChooseReplacementOrder { .. }
                    | Action::OrderDamageAssignment { .. }
            ) {
                return action.clone();
            }
        }

        // Forced discard during cleanup: discard the first card
        for action in &actions {
            if let Action::Discard { .. } = action {
                return action.clone();
            }
        }

        // Declare attackers: never attack (pick the empty attacker set)
        if matches!(state.phase, crate::game::Phase::DeclareAttackers) && player == state.active_player {
            for action in &actions {
                if let Action::DeclareAttackers { attackers } = action {
                    if attackers.is_empty() {
                        return action.clone();
                    }
                }
            }
        }

        // Declare blockers: never block (pick the empty block set)
        if matches!(state.phase, crate::game::Phase::DeclareBlockers) && player != state.active_player {
            for action in &actions {
                if let Action::DeclareBlockers { blocks } = action {
                    if blocks.is_empty() {
                        return action.clone();
                    }
                }
            }
        }

        // Default: always pass priority
        Action::PassPriority
    }

    fn name(&self) -> &str {
        "Goldfish"
    }
}

/// Evaluate a blocking assignment. Positive = good for the defender.
fn evaluate_blocks(state: &GameState, blocks: &[(ObjectId, ObjectId)]) -> i32 {
    let db = state.card_db();
    let mut score: i32 = 0;

    for &(blocker_id, attacker_id) in blocks {
        let blocker_inst = &state.objects[&blocker_id];
        let attacker_inst = &state.objects[&attacker_id];

        let blocker_def = match db.get(blocker_inst.card_def_id) {
            Some(d) => d,
            None => continue,
        };
        let attacker_def = match db.get(attacker_inst.card_def_id) {
            Some(d) => d,
            None => continue,
        };

        let attacker_power = state.effective_power(attacker_id);
        let attacker_toughness = state.effective_toughness(attacker_id);
        let blocker_power = state.effective_power(blocker_id);
        let blocker_toughness = state.effective_toughness(blocker_id);

        let blocker_dies = attacker_power >= blocker_toughness;
        let attacker_dies = blocker_power >= attacker_toughness;

        // Prevented damage (the attacker's power that won't hit the player)
        score += attacker_power;

        if attacker_dies && !blocker_dies {
            // We kill their creature and ours survives — great trade
            score += attacker_def.cmc() as i32 * 2;
        } else if attacker_dies && blocker_dies {
            // Trade — good if their creature is more expensive
            score += (attacker_def.cmc() as i32) - (blocker_def.cmc() as i32);
        } else if !attacker_dies && blocker_dies {
            // We lose our creature, they keep theirs — bad unless preventing lethal
            score -= blocker_def.cmc() as i32;
        }
    }

    score
}

/// Lookahead strategy: 1-ply search with greedy rollout evaluation.
///
/// At each decision point with multiple legal actions, this strategy:
/// 1. Tries each candidate action
/// 2. Plays out the next few turns with GreedyStrategy (for the pilot)
///    and GoldfishStrategy (for the opponent)
/// 3. Evaluates the resulting board position
/// 4. Picks the action that leads to the best outcome
///
/// This is much stronger than pure Greedy because it can evaluate
/// multi-step consequences (e.g., "play land now so I can cast creature next turn"
/// vs "cast a cheaper creature now").
///
/// For single-action or forced decisions, falls back to Greedy for speed.
pub struct LookaheadStrategy {
    /// Number of turns to simulate in the rollout.
    pub rollout_turns: u32,
    /// Maximum actions in the rollout before stopping.
    pub rollout_max_actions: u32,
}

impl Default for LookaheadStrategy {
    fn default() -> Self {
        LookaheadStrategy {
            rollout_turns: 4,
            rollout_max_actions: 150,
        }
    }
}

impl LookaheadStrategy {
    /// Evaluate a game state by rolling out with greedy play for a few turns.
    /// Returns a score in [-1, 1] where higher is better for `pilot`.
    fn rollout_evaluate(&self, state: &GameState, pilot: PlayerIndex) -> f64 {
        use crate::rules;

        if state.game_over {
            return if state.winner == Some(pilot) { 1.0 } else { -1.0 };
        }

        let mut s = state.clone();
        let greedy = GreedyStrategy;
        let goldfish = GoldfishStrategy;
        let end_turn = s.turn_number + self.rollout_turns;
        let mut actions_taken = 0u32;

        while !s.game_over && s.turn_number <= end_turn && actions_taken < self.rollout_max_actions {
            let player = s.priority_player;
            let action = if player == pilot {
                greedy.choose_action(&s, player)
            } else {
                goldfish.choose_action(&s, player)
            };
            rules::apply_action(&mut s, &action);
            actions_taken += 1;
        }

        if s.game_over {
            if s.winner == Some(pilot) { 1.0 } else { -1.0 }
        } else {
            // Heuristic score based on board state
            self.heuristic_score(&s, pilot)
        }
    }

    /// Static evaluation of a board state. Higher = better for pilot.
    fn heuristic_score(&self, state: &GameState, pilot: PlayerIndex) -> f64 {
        let opp = 1 - pilot;
        let starting_life = match state.format {
            crate::game::GameFormat::Commander => 40.0,
            _ => 20.0,
        };
        let opp_life = state.players[opp].life as f64;

        // Damage dealt (most important)
        let damage_frac = ((starting_life - opp_life) / starting_life).clamp(0.0, 1.0);
        let damage_score = damage_frac.sqrt();

        // Board power
        let my_power: i32 = state.creatures_controlled_by(pilot)
            .iter()
            .map(|&id| state.effective_power(id))
            .sum();
        let power_score = (my_power as f64 / 15.0).clamp(0.0, 1.0);

        // Lands in play
        let db = state.card_db();
        let lands_in_play = state.battlefield.iter()
            .filter(|&&obj_id| {
                if let Some(inst) = state.objects.get(&obj_id) {
                    inst.controller == pilot
                        && db.get(inst.card_def_id).map_or(false, |d| d.is_land())
                } else {
                    false
                }
            })
            .count() as f64;
        let mana_score = (lands_in_play / 5.0).clamp(0.0, 1.0);

        // Non-land permanents
        let nonland_perms = state.battlefield.iter()
            .filter(|&&obj_id| {
                if let Some(inst) = state.objects.get(&obj_id) {
                    inst.controller == pilot
                        && !db.get(inst.card_def_id).map_or(true, |d| d.is_land())
                } else {
                    false
                }
            })
            .count() as f64;
        let perm_score = (nonland_perms / 4.0).clamp(0.0, 1.0);

        damage_score * 0.35 + power_score * 0.30 + mana_score * 0.20 + perm_score * 0.15
    }
}

impl Strategy for LookaheadStrategy {
    fn choose_action(&self, state: &GameState, player: PlayerIndex) -> Action {
        use crate::action::legal_actions_abstracted;

        let actions = legal_actions_abstracted(state);
        if actions.is_empty() {
            return Action::PassPriority;
        }
        if actions.len() == 1 {
            return actions[0].clone();
        }

        // For forced/mechanical decisions, use Greedy for speed
        for action in &actions {
            if matches!(action, Action::OrderTriggers { .. } | Action::ChooseReplacementOrder { .. }) {
                return action.clone();
            }
        }
        if state.phase == crate::game::Phase::Mulligan {
            return GreedyStrategy.choose_action(state, player);
        }

        // 1-ply lookahead: try each action and evaluate
        let mut best_score = f64::NEG_INFINITY;
        let mut best_action = actions[0].clone();

        for action in &actions {
            let mut child = state.clone();
            crate::rules::apply_action(&mut child, action);
            let score = self.rollout_evaluate(&child, player);
            if score > best_score {
                best_score = score;
                best_action = action.clone();
            }
        }

        best_action
    }

    fn name(&self) -> &str {
        "Lookahead"
    }
}
