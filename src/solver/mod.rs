//! Phase 1B.2 — MCCFR Solver Infrastructure
//!
//! Contains the regret table data structure and the MCCFR traversal algorithm.
//! The solver consumes the game engine as a black box through:
//! - `legal_actions_abstracted()` — bounded action enumeration
//! - `canonicalize()` / `resolve()` — stable action identifiers
//! - `apply_action()` — deterministic state transitions
//! - `visible_state()` — information set boundary
//!
//! # Architecture
//!
//! - `RegretTable` — stores cumulative regret and cumulative strategy per info set,
//!   keyed by `(info_set_hash, CanonicalAction)` for stable action identification
//! - `McfrStrategy` — implements `Strategy` trait using a trained regret table
//! - `mccfr` submodule — external sampling MCCFR traversal

pub mod mccfr;

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::action::canonical::CanonicalAction;

/// Per-action regret and strategy accumulation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionEntry {
    pub cumulative_regret: f64,
    pub cumulative_strategy: f64,
}

/// Per-information-set data stored in the regret table.
///
/// Actions are keyed by `CanonicalAction` (stable identifiers independent of
/// ObjectId assignment) rather than positional index. This ensures that the
/// same action at the same information set always maps to the same regret/
/// strategy slot, even if `legal_actions_abstracted()` returns actions in a
/// different order across visits.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InfoSetData {
    /// Per-action regret and strategy data, keyed by canonical action.
    pub action_data: HashMap<CanonicalAction, ActionEntry>,
    /// Number of times this info set has been visited.
    pub visit_count: u64,
}

impl InfoSetData {
    /// Create a new empty entry.
    pub fn new() -> Self {
        InfoSetData {
            action_data: HashMap::new(),
            visit_count: 0,
        }
    }

    /// Compute the current strategy via regret matching over the given actions.
    ///
    /// Actions with positive cumulative regret get probability proportional
    /// to their regret. If all regrets are non-positive, play uniformly.
    /// Returns probabilities in the same order as the input `actions` slice.
    pub fn current_strategy(&self, actions: &[CanonicalAction]) -> Vec<f64> {
        let n = actions.len();
        if n == 0 {
            return vec![];
        }

        let regrets: Vec<f64> = actions
            .iter()
            .map(|a| {
                self.action_data
                    .get(a)
                    .map(|d| d.cumulative_regret)
                    .unwrap_or(0.0)
            })
            .collect();

        let positive_sum: f64 = regrets.iter().filter(|&&r| r > 0.0).sum();

        if positive_sum > 0.0 {
            regrets
                .iter()
                .map(|&r| if r > 0.0 { r / positive_sum } else { 0.0 })
                .collect()
        } else {
            let uniform = 1.0 / n as f64;
            vec![uniform; n]
        }
    }

    /// Compute the average strategy (converges to Nash equilibrium).
    ///
    /// This is what should be used for play after training, not the
    /// current strategy (which oscillates during training).
    /// Returns probabilities in the same order as the input `actions` slice.
    pub fn average_strategy(&self, actions: &[CanonicalAction]) -> Vec<f64> {
        let n = actions.len();
        if n == 0 {
            return vec![];
        }

        let strats: Vec<f64> = actions
            .iter()
            .map(|a| {
                self.action_data
                    .get(a)
                    .map(|d| d.cumulative_strategy)
                    .unwrap_or(0.0)
            })
            .collect();

        let total: f64 = strats.iter().sum();
        if total > 0.0 {
            strats.iter().map(|&s| s / total).collect()
        } else {
            let uniform = 1.0 / n as f64;
            vec![uniform; n]
        }
    }

    /// Get or create the entry for a canonical action.
    pub fn get_or_create_action(&mut self, action: &CanonicalAction) -> &mut ActionEntry {
        self.action_data.entry(action.clone()).or_insert(ActionEntry {
            cumulative_regret: 0.0,
            cumulative_strategy: 0.0,
        })
    }
}

impl Default for InfoSetData {
    fn default() -> Self {
        Self::new()
    }
}

/// Regret table: maps information set hashes to per-action regret data.
///
/// This is the core data structure of MCCFR. Each entry corresponds to
/// a unique information set (observable game state from one player's
/// perspective) and stores the cumulative regret and strategy weights
/// for each available action at that info set. Actions are identified
/// by `CanonicalAction` for stability across different game instances.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegretTable {
    /// Map from information set hash to per-action data.
    pub data: HashMap<u64, InfoSetData>,
}

impl RegretTable {
    /// Create an empty regret table.
    pub fn new() -> Self {
        RegretTable {
            data: HashMap::new(),
        }
    }

    /// Get or create the entry for an information set.
    pub fn get_or_create(&mut self, info_set_hash: u64) -> &mut InfoSetData {
        self.data
            .entry(info_set_hash)
            .or_insert_with(InfoSetData::new)
    }

    /// Get the entry for an information set, if it exists.
    pub fn get(&self, info_set_hash: u64) -> Option<&InfoSetData> {
        self.data.get(&info_set_hash)
    }

    /// Number of unique information sets stored.
    pub fn num_info_sets(&self) -> usize {
        self.data.len()
    }

    /// Prune entries that have never been visited more than `min_visits` times.
    /// Useful for controlling memory usage on large training runs.
    pub fn prune(&mut self, min_visits: u64) {
        self.data.retain(|_, v| v.visit_count >= min_visits);
    }

    /// Serialize to bytes using bincode.
    pub fn to_bytes(&self) -> Result<Vec<u8>, bincode::Error> {
        bincode::serialize(self)
    }

    /// Deserialize from bytes using bincode.
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, bincode::Error> {
        bincode::deserialize(bytes)
    }
}

impl Default for RegretTable {
    fn default() -> Self {
        Self::new()
    }
}

/// Sample an action index from a probability distribution.
///
/// Shared utility used by both the MCCFR traversal and McfrStrategy.
pub fn sample_from_distribution(distribution: &[f64], rng: &mut impl rand::Rng) -> usize {
    let r: f64 = rng.gen();
    let mut cumulative = 0.0;
    for (i, &p) in distribution.iter().enumerate() {
        cumulative += p;
        if r < cumulative {
            return i;
        }
    }
    // Fallback to last action (rounding errors)
    distribution.len().saturating_sub(1)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_info_set_data_current_strategy_uniform() {
        let data = InfoSetData::new();
        let actions = vec![
            CanonicalAction::PassPriority,
            CanonicalAction::Concede,
            CanonicalAction::PlayLand { card_id: 1, hand_index: 0 },
        ];
        let strategy = data.current_strategy(&actions);
        assert_eq!(strategy.len(), 3);
        for &p in &strategy {
            assert!((p - 1.0 / 3.0).abs() < 1e-10);
        }
    }

    #[test]
    fn test_info_set_data_current_strategy_regret_matching() {
        let mut data = InfoSetData::new();
        let a0 = CanonicalAction::PassPriority;
        let a1 = CanonicalAction::Concede;
        let a2 = CanonicalAction::PlayLand { card_id: 1, hand_index: 0 };
        data.get_or_create_action(&a0).cumulative_regret = 10.0;
        data.get_or_create_action(&a1).cumulative_regret = 0.0;
        data.get_or_create_action(&a2).cumulative_regret = -5.0;

        let strategy = data.current_strategy(&[a0, a1, a2]);
        assert!((strategy[0] - 1.0).abs() < 1e-10); // only positive regret
        assert!((strategy[1]).abs() < 1e-10);
        assert!((strategy[2]).abs() < 1e-10);
    }

    #[test]
    fn test_info_set_data_current_strategy_multiple_positive() {
        let mut data = InfoSetData::new();
        let a0 = CanonicalAction::PassPriority;
        let a1 = CanonicalAction::Concede;
        let a2 = CanonicalAction::PlayLand { card_id: 1, hand_index: 0 };
        data.get_or_create_action(&a0).cumulative_regret = 6.0;
        data.get_or_create_action(&a1).cumulative_regret = 4.0;
        data.get_or_create_action(&a2).cumulative_regret = -2.0;

        let strategy = data.current_strategy(&[a0, a1, a2]);
        assert!((strategy[0] - 0.6).abs() < 1e-10);
        assert!((strategy[1] - 0.4).abs() < 1e-10);
        assert!((strategy[2]).abs() < 1e-10);
    }

    #[test]
    fn test_info_set_data_average_strategy() {
        let mut data = InfoSetData::new();
        let a0 = CanonicalAction::PassPriority;
        let a1 = CanonicalAction::Concede;
        let a2 = CanonicalAction::PlayLand { card_id: 1, hand_index: 0 };
        data.get_or_create_action(&a0).cumulative_strategy = 100.0;
        data.get_or_create_action(&a1).cumulative_strategy = 200.0;
        data.get_or_create_action(&a2).cumulative_strategy = 300.0;

        let avg = data.average_strategy(&[a0, a1, a2]);
        assert!((avg[0] - 1.0 / 6.0).abs() < 1e-10);
        assert!((avg[1] - 2.0 / 6.0).abs() < 1e-10);
        assert!((avg[2] - 3.0 / 6.0).abs() < 1e-10);
    }

    #[test]
    fn test_regret_table_get_or_create() {
        let mut table = RegretTable::new();
        let a0 = CanonicalAction::PassPriority;
        {
            let data = table.get_or_create(42);
            data.get_or_create_action(&a0).cumulative_regret = 5.0;
            data.visit_count = 1;
        }
        assert_eq!(table.num_info_sets(), 1);
        assert!(table.get(42).is_some());
        assert_eq!(
            table.get(42).unwrap().action_data[&a0].cumulative_regret,
            5.0
        );
    }

    #[test]
    fn test_regret_table_serialization_roundtrip() {
        let mut table = RegretTable::new();
        let actions = vec![
            CanonicalAction::PassPriority,
            CanonicalAction::Concede,
            CanonicalAction::PlayLand { card_id: 1, hand_index: 0 },
            CanonicalAction::CastSpell { card_id: 100, hand_index: 0, targets: vec![] },
        ];
        {
            let data = table.get_or_create(100);
            for (i, a) in actions.iter().enumerate() {
                let entry = data.get_or_create_action(a);
                entry.cumulative_regret = (i + 1) as f64;
                entry.cumulative_strategy = ((i + 1) * 10) as f64;
            }
            data.visit_count = 50;
        }

        let bytes = table.to_bytes().expect("serialization failed");
        let restored = RegretTable::from_bytes(&bytes).expect("deserialization failed");

        assert_eq!(restored.num_info_sets(), 1);
        let data = restored.get(100).unwrap();
        assert_eq!(data.action_data[&CanonicalAction::PassPriority].cumulative_regret, 1.0);
        assert_eq!(data.action_data[&CanonicalAction::Concede].cumulative_strategy, 20.0);
        assert_eq!(data.visit_count, 50);
    }

    #[test]
    fn test_regret_table_prune() {
        let mut table = RegretTable::new();
        table.get_or_create(1).visit_count = 100;
        table.get_or_create(2).visit_count = 1;
        table.get_or_create(3).visit_count = 50;

        assert_eq!(table.num_info_sets(), 3);
        table.prune(10);
        assert_eq!(table.num_info_sets(), 2);
        assert!(table.get(1).is_some());
        assert!(table.get(2).is_none());
        assert!(table.get(3).is_some());
    }

    #[test]
    fn test_sample_from_distribution() {
        let dist = vec![0.5, 0.3, 0.2];
        let mut rng = rand::thread_rng();
        let mut counts = vec![0u32; 3];
        for _ in 0..1000 {
            let idx = sample_from_distribution(&dist, &mut rng);
            assert!(idx < 3);
            counts[idx] += 1;
        }
        for &c in &counts {
            assert!(c > 0, "All actions should be sampled");
        }
    }
}
