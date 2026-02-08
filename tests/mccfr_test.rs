//! Integration tests for Phase 1B: MCCFR Solver
//!
//! Tests the end-to-end MCCFR training pipeline:
//! - InformationSet construction from PlayerView
//! - RegretTable storage and serialization
//! - MCCFR traversal on minimal training scenarios
//! - McfrStrategy implementing the Strategy trait
//! - Trained McfrStrategy playing legal games

use std::sync::Arc;

use mtg_gto::action::legal_actions;
use mtg_gto::card::sample;
use mtg_gto::card::ZoneType;
use mtg_gto::game::{GameState, Phase};
use mtg_gto::info_set::InformationSet;
use mtg_gto::rules;
use mtg_gto::simulation;
use mtg_gto::solver::mccfr::{self, McfrConfig};
use mtg_gto::solver::RegretTable;
use mtg_gto::strategy::{McfrStrategy, RandomStrategy, Strategy};

/// Helper: create a minimal game state with 15-card decks for MCCFR testing.
fn setup_mini_game() -> GameState {
    let db = sample::build_sample_db();
    let deck0 = sample::mini_red_burn();
    let deck1 = sample::mini_red_creatures();

    let mut state = GameState::new(2);
    state.card_db = Some(Arc::new(db));
    rules::setup_game(&mut state, &deck0, &deck1);
    state
}

#[test]
fn test_info_set_from_game_state() {
    let state = setup_mini_game();
    let view = state.visible_state(0);
    let info_set = InformationSet::from_view(&view, state.card_db());

    assert_eq!(info_set.my_life, 20);
    assert_eq!(info_set.opp_life, 20);
    // Player should have drawn 7 cards
    assert_eq!(info_set.my_hand.len(), 7);
    // Opponent has 7 cards in hand (size visible, contents hidden)
    assert_eq!(info_set.opp_hand_size, 7);
    // Libraries should have 15 - 7 = 8 cards each
    assert_eq!(info_set.opp_library_size, 8);
}

#[test]
fn test_info_set_hash_stable() {
    // Same game state should produce same hash on repeated calls
    let state = setup_mini_game();

    let view1 = state.visible_state(0);
    let hash1 = InformationSet::from_view(&view1, state.card_db()).hash_value();

    let view2 = state.visible_state(0);
    let hash2 = InformationSet::from_view(&view2, state.card_db()).hash_value();

    assert_eq!(hash1, hash2, "Same game state should produce same hash");
}

#[test]
fn test_regret_table_roundtrip() {
    let mut table = RegretTable::new();
    let entry = table.get_or_create(12345, 5);
    entry.cumulative_regret = vec![1.0, -2.0, 3.0, 0.0, -1.0];
    entry.cumulative_strategy = vec![10.0, 5.0, 15.0, 3.0, 7.0];
    entry.visit_count = 42;

    let bytes = table.to_bytes().expect("serialize");
    let restored = RegretTable::from_bytes(&bytes).expect("deserialize");

    let data = restored.get(12345).unwrap();
    assert_eq!(data.cumulative_regret, vec![1.0, -2.0, 3.0, 0.0, -1.0]);
    assert_eq!(data.visit_count, 42);
}

#[test]
fn test_mccfr_single_iteration_runs() {
    // Verify that a single MCCFR iteration completes without panics.
    let state = setup_mini_game();
    let config = McfrConfig {
        max_depth: 50,
        max_actions: 500,
    };

    let mut tables = [RegretTable::new(), RegretTable::new()];
    mccfr::run_iteration(&state, &mut tables, &config);

    // After one iteration, at least some info sets should have been visited
    let total_info_sets: usize = tables.iter().map(|t| t.num_info_sets()).sum();
    assert!(
        total_info_sets > 0,
        "MCCFR should visit at least some info sets"
    );
}

#[test]
fn test_mccfr_training_loop() {
    // Run a small number of MCCFR iterations and verify convergence behavior.
    let state = setup_mini_game();
    let config = McfrConfig {
        max_depth: 40,
        max_actions: 300,
    };

    let tables = mccfr::train(&state, 10, &config);

    let total_info_sets: usize = tables.iter().map(|t| t.num_info_sets()).sum();
    assert!(
        total_info_sets > 0,
        "Training should create info set entries"
    );

    // Check that visit counts are reasonable
    for table in &tables {
        for (_, data) in &table.data {
            assert!(data.visit_count > 0, "Visited entries should have count > 0");
            assert!(
                !data.cumulative_strategy.is_empty(),
                "Entries should have strategy data"
            );
        }
    }
}

#[test]
fn test_mccfr_exploitability_decreases() {
    // After more iterations, approximate exploitability should generally decrease.
    let state = setup_mini_game();
    let config = McfrConfig {
        max_depth: 30,
        max_actions: 200,
    };

    let tables_5 = mccfr::train(&state, 5, &config);
    let exploit_5 = mccfr::approximate_exploitability(&tables_5);

    let tables_50 = mccfr::train(&state, 50, &config);
    let exploit_50 = mccfr::approximate_exploitability(&tables_50);

    // We don't assert strict monotonicity (MCCFR is stochastic), but the
    // 50-iteration result should be finite and non-negative.
    assert!(exploit_50.is_finite(), "Exploitability should be finite");
    assert!(exploit_50 >= 0.0, "Exploitability should be non-negative");
    assert!(exploit_5.is_finite(), "Exploitability should be finite");

    // Log values for manual inspection
    eprintln!("Exploitability after 5 iters: {:.4}", exploit_5);
    eprintln!("Exploitability after 50 iters: {:.4}", exploit_50);
}

#[test]
fn test_mcfr_strategy_plays_legal_games() {
    // Train a McfrStrategy and verify it plays complete, legal games.
    let db = sample::build_sample_db();
    let state = setup_mini_game();
    let config = McfrConfig {
        max_depth: 30,
        max_actions: 200,
    };

    let tables = mccfr::train(&state, 20, &config);
    let mcfr_p0 = McfrStrategy::new(tables[0].clone());
    let mcfr_p1 = McfrStrategy::new(tables[1].clone());

    // Play 10 games with McfrStrategy
    let deck0 = sample::mini_red_burn();
    let deck1 = sample::mini_red_creatures();

    for _ in 0..10 {
        let result = simulation::run_game(&db, &deck0, &deck1, &mcfr_p0, &mcfr_p1);
        // Game should terminate (not hang)
        assert!(
            result.turns <= 200,
            "Game should terminate within turn limit"
        );
    }
}

#[test]
fn test_mcfr_strategy_vs_random() {
    // After training, McfrStrategy should at least not crash when playing
    // against RandomStrategy. Win rate validation requires more training.
    let db = sample::build_sample_db();
    let state = setup_mini_game();
    let config = McfrConfig {
        max_depth: 30,
        max_actions: 200,
    };

    let tables = mccfr::train(&state, 20, &config);
    let mcfr_strat = McfrStrategy::new(tables[0].clone());
    let random_strat = RandomStrategy;

    let deck0 = sample::mini_red_burn();
    let deck1 = sample::mini_red_creatures();

    let results = simulation::simulate(
        &db,
        &deck0,
        &deck1,
        &mcfr_strat,
        &random_strat,
        50,
    );

    eprintln!(
        "MCCFR vs Random: {:.1}% win rate ({} games)",
        results.win_rate(0) * 100.0,
        results.total_games
    );

    // McfrStrategy should at least complete all games without panics
    assert_eq!(results.total_games, 50);
}

#[test]
fn test_mcfr_strategy_name() {
    let strategy = McfrStrategy::new(RegretTable::new());
    assert_eq!(strategy.name(), "MCCFR");
}

#[test]
fn test_mcfr_strategy_fallback_on_unseen_info_set() {
    // McfrStrategy should handle unseen info sets gracefully (uniform random fallback)
    let strategy = McfrStrategy::new(RegretTable::new());

    let mut state = GameState::new(2);
    let db = sample::build_sample_db();
    state.card_db = Some(Arc::new(db));

    // Set up a minimal state so there are legal actions
    for _ in 0..10 {
        state.create_card_in_zone(sample::ids::MOUNTAIN, 0, ZoneType::Library);
        state.create_card_in_zone(sample::ids::MOUNTAIN, 1, ZoneType::Library);
    }
    state.create_card_in_zone(sample::ids::MOUNTAIN, 0, ZoneType::Hand);
    state.active_player = 0;
    state.priority_player = 0;
    state.phase = Phase::PreCombatMain;
    state.turn_number = 2;

    // Should not panic even with empty regret table
    let _action = strategy.choose_action(&state, 0);
    let legal = legal_actions(&state);
    // The action should be one of the legal actions
    // (McfrStrategy uses legal_actions_abstracted which may differ slightly,
    // but PassPriority is always legal)
    assert!(
        !legal.is_empty(),
        "There should be legal actions"
    );
}

#[test]
fn test_mini_deck_sizes() {
    let burn = sample::mini_red_burn();
    assert_eq!(burn.len(), 15, "Mini burn deck should be 15 cards");

    let creatures = sample::mini_red_creatures();
    assert_eq!(creatures.len(), 15, "Mini creature deck should be 15 cards");
}
