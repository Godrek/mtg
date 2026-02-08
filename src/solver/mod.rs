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
//! - `RegretTable` — stores cumulative regret and cumulative strategy per info set
//! - `McfrStrategy` — implements `Strategy` trait using a trained regret table
//! - `mccfr` submodule — external sampling MCCFR traversal

pub mod mccfr;

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

/// Per-action data stored for each information set.
///
/// Cumulative regret drives the current strategy via regret matching.
/// Cumulative strategy tracks the average strategy across iterations
/// (which converges to Nash equilibrium).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InfoSetData {
    /// Cumulative counterfactual regret for each action.
    /// Indexed by position in the canonical action list.
    pub cumulative_regret: Vec<f64>,
    /// Cumulative strategy weight for each action (for computing average strategy).
    pub cumulative_strategy: Vec<f64>,
    /// Number of times this info set has been visited.
    pub visit_count: u64,
}

impl InfoSetData {
    /// Create a new entry with `num_actions` slots.
    pub fn new(num_actions: usize) -> Self {
        InfoSetData {
            cumulative_regret: vec![0.0; num_actions],
            cumulative_strategy: vec![0.0; num_actions],
            visit_count: 0,
        }
    }

    /// Compute the current strategy via regret matching.
    ///
    /// Actions with positive cumulative regret get probability proportional
    /// to their regret. If all regrets are non-positive, play uniformly.
    pub fn current_strategy(&self) -> Vec<f64> {
        let n = self.cumulative_regret.len();
        if n == 0 {
            return vec![];
        }

        let positive_sum: f64 = self
            .cumulative_regret
            .iter()
            .filter(|&&r| r > 0.0)
            .sum();

        if positive_sum > 0.0 {
            self.cumulative_regret
                .iter()
                .map(|&r| if r > 0.0 { r / positive_sum } else { 0.0 })
                .collect()
        } else {
            // Uniform distribution when no action has positive regret
            let uniform = 1.0 / n as f64;
            vec![uniform; n]
        }
    }

    /// Compute the average strategy (converges to Nash equilibrium).
    ///
    /// This is what should be used for play after training, not the
    /// current strategy (which oscillates during training).
    pub fn average_strategy(&self) -> Vec<f64> {
        let n = self.cumulative_strategy.len();
        if n == 0 {
            return vec![];
        }

        let total: f64 = self.cumulative_strategy.iter().sum();
        if total > 0.0 {
            self.cumulative_strategy
                .iter()
                .map(|&s| s / total)
                .collect()
        } else {
            let uniform = 1.0 / n as f64;
            vec![uniform; n]
        }
    }
}

/// Regret table: maps information set hashes to per-action regret data.
///
/// This is the core data structure of MCCFR. Each entry corresponds to
/// a unique information set (observable game state from one player's
/// perspective) and stores the cumulative regret and strategy weights
/// for each available action at that info set.
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
    ///
    /// If the info set hasn't been seen before, creates a new entry
    /// with `num_actions` slots. If it exists but has a different
    /// number of actions (shouldn't happen in practice), the existing
    /// entry is returned as-is.
    pub fn get_or_create(&mut self, info_set_hash: u64, num_actions: usize) -> &mut InfoSetData {
        self.data
            .entry(info_set_hash)
            .or_insert_with(|| InfoSetData::new(num_actions))
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_info_set_data_current_strategy_uniform() {
        let data = InfoSetData::new(3);
        let strategy = data.current_strategy();
        assert_eq!(strategy.len(), 3);
        for &p in &strategy {
            assert!((p - 1.0 / 3.0).abs() < 1e-10);
        }
    }

    #[test]
    fn test_info_set_data_current_strategy_regret_matching() {
        let mut data = InfoSetData::new(3);
        data.cumulative_regret = vec![10.0, 0.0, -5.0];
        let strategy = data.current_strategy();
        assert!((strategy[0] - 1.0).abs() < 1e-10); // only positive regret
        assert!((strategy[1]).abs() < 1e-10);
        assert!((strategy[2]).abs() < 1e-10);
    }

    #[test]
    fn test_info_set_data_current_strategy_multiple_positive() {
        let mut data = InfoSetData::new(3);
        data.cumulative_regret = vec![6.0, 4.0, -2.0];
        let strategy = data.current_strategy();
        assert!((strategy[0] - 0.6).abs() < 1e-10);
        assert!((strategy[1] - 0.4).abs() < 1e-10);
        assert!((strategy[2]).abs() < 1e-10);
    }

    #[test]
    fn test_info_set_data_average_strategy() {
        let mut data = InfoSetData::new(3);
        data.cumulative_strategy = vec![100.0, 200.0, 300.0];
        let avg = data.average_strategy();
        assert!((avg[0] - 1.0 / 6.0).abs() < 1e-10);
        assert!((avg[1] - 2.0 / 6.0).abs() < 1e-10);
        assert!((avg[2] - 3.0 / 6.0).abs() < 1e-10);
    }

    #[test]
    fn test_regret_table_get_or_create() {
        let mut table = RegretTable::new();
        {
            let data = table.get_or_create(42, 3);
            data.cumulative_regret[0] = 5.0;
            data.visit_count = 1;
        }
        assert_eq!(table.num_info_sets(), 1);
        assert!(table.get(42).is_some());
        assert_eq!(table.get(42).unwrap().cumulative_regret[0], 5.0);
    }

    #[test]
    fn test_regret_table_serialization_roundtrip() {
        let mut table = RegretTable::new();
        {
            let data = table.get_or_create(100, 4);
            data.cumulative_regret = vec![1.0, 2.0, 3.0, 4.0];
            data.cumulative_strategy = vec![10.0, 20.0, 30.0, 40.0];
            data.visit_count = 50;
        }

        let bytes = table.to_bytes().expect("serialization failed");
        let restored = RegretTable::from_bytes(&bytes).expect("deserialization failed");

        assert_eq!(restored.num_info_sets(), 1);
        let data = restored.get(100).unwrap();
        assert_eq!(data.cumulative_regret, vec![1.0, 2.0, 3.0, 4.0]);
        assert_eq!(data.cumulative_strategy, vec![10.0, 20.0, 30.0, 40.0]);
        assert_eq!(data.visit_count, 50);
    }

    #[test]
    fn test_regret_table_prune() {
        let mut table = RegretTable::new();
        table.get_or_create(1, 2).visit_count = 100;
        table.get_or_create(2, 2).visit_count = 1;
        table.get_or_create(3, 2).visit_count = 50;

        assert_eq!(table.num_info_sets(), 3);
        table.prune(10);
        assert_eq!(table.num_info_sets(), 2);
        assert!(table.get(1).is_some());
        assert!(table.get(2).is_none());
        assert!(table.get(3).is_some());
    }
}
