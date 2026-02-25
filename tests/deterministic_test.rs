//! Phase 13.4: Deterministic replay tests.
//!
//! Verifies that seeded games produce identical results when replayed with
//! the same seed, enabling reproducible debugging.

use mtg_gto::card::sample;
use mtg_gto::simulation;
use mtg_gto::strategy::GreedyStrategy;

#[test]
fn test_seeded_game_produces_same_result() {
    let db = sample::build_sample_db();
    let red = sample::red_aggro_deck();
    let green = sample::green_stompy_deck();
    let greedy = GreedyStrategy;
    let seed = 42;

    let result1 = simulation::run_game_seeded(&db, &red, &green, &greedy, &greedy, seed);
    let result2 = simulation::run_game_seeded(&db, &red, &green, &greedy, &greedy, seed);

    assert_eq!(
        result1.winner, result2.winner,
        "Same seed should produce same winner"
    );
    assert_eq!(
        result1.turns, result2.turns,
        "Same seed should produce same number of turns"
    );
    assert_eq!(
        result1.actions_taken, result2.actions_taken,
        "Same seed should produce same number of actions"
    );
    assert_eq!(
        result1.final_life, result2.final_life,
        "Same seed should produce same final life totals"
    );
}

#[test]
fn test_different_seeds_produce_different_results() {
    let db = sample::build_sample_db();
    let red = sample::red_aggro_deck();
    let green = sample::green_stompy_deck();
    let greedy = GreedyStrategy;

    // Run with multiple seeds and check that at least some differ
    let mut results = Vec::new();
    for seed in 0..20 {
        let result = simulation::run_game_seeded(&db, &red, &green, &greedy, &greedy, seed);
        results.push(result);
    }

    // At least some games should have different outcomes
    let all_same_turns = results.iter().all(|r| r.turns == results[0].turns);
    assert!(
        !all_same_turns,
        "Different seeds should produce at least some different game lengths"
    );
}

#[test]
fn test_seeded_commander_goldfish_reproducible() {
    let db = sample::build_sample_db();
    let (deck, commander) = sample::brimaz_commander_deck();
    let greedy = GreedyStrategy;
    let seed = 12345;

    let result1 =
        simulation::run_commander_goldfish_game_seeded(&db, &deck, commander, &greedy, seed);
    let result2 =
        simulation::run_commander_goldfish_game_seeded(&db, &deck, commander, &greedy, seed);

    assert_eq!(
        result1.winner, result2.winner,
        "Seeded commander goldfish should be reproducible (winner)"
    );
    assert_eq!(
        result1.turns, result2.turns,
        "Seeded commander goldfish should be reproducible (turns)"
    );
    assert_eq!(
        result1.actions_taken, result2.actions_taken,
        "Seeded commander goldfish should be reproducible (actions)"
    );
}

#[test]
fn test_seeded_mirror_match_reproducible() {
    let db = sample::build_sample_db();
    let red = sample::red_aggro_deck();
    let greedy = GreedyStrategy;
    let seed = 99999;

    let result1 = simulation::run_game_seeded(&db, &red, &red, &greedy, &greedy, seed);
    let result2 = simulation::run_game_seeded(&db, &red, &red, &greedy, &greedy, seed);

    assert_eq!(result1.winner, result2.winner);
    assert_eq!(result1.turns, result2.turns);
    assert_eq!(result1.final_life, result2.final_life);
}
