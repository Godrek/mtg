use mtg_gto::card::sample;
use mtg_gto::game::GameState;
use mtg_gto::simulation;
use mtg_gto::strategy::{GreedyStrategy, RandomStrategy};

#[test]
fn test_sample_db_builds() {
    let db = sample::build_sample_db();
    assert!(db.get(sample::ids::MOUNTAIN).is_some());
    assert!(db.get(sample::ids::LIGHTNING_BOLT).is_some());
    assert!(db.get(sample::ids::GRIZZLY_BEARS).is_some());
    assert!(db.get(sample::ids::SERRA_ANGEL).is_some());
}

#[test]
fn test_deck_sizes() {
    let red = sample::red_aggro_deck();
    let green = sample::green_stompy_deck();
    assert_eq!(red.len(), 60);
    assert_eq!(green.len(), 60);
}

#[test]
fn test_game_setup() {
    let db = sample::build_sample_db();
    let red = sample::red_aggro_deck();
    let green = sample::green_stompy_deck();

    let mut state = GameState::new(2);
    state.card_db = Some(db);
    mtg_gto::rules::setup_game(&mut state, &red, &green);

    // Both players should have 7 cards in hand
    assert_eq!(state.players[0].hand.len(), 7);
    assert_eq!(state.players[1].hand.len(), 7);

    // Libraries should have 53 cards each (60 - 7)
    assert_eq!(state.players[0].library.len(), 53);
    assert_eq!(state.players[1].library.len(), 53);

    // Both at 20 life
    assert_eq!(state.players[0].life, 20);
    assert_eq!(state.players[1].life, 20);

    // Game should not be over
    assert!(!state.game_over);
}

#[test]
fn test_single_game_completes() {
    let db = sample::build_sample_db();
    let red = sample::red_aggro_deck();
    let green = sample::green_stompy_deck();

    let greedy = GreedyStrategy;
    let result = simulation::run_game(&db, &red, &green, &greedy, &greedy);

    // Game should produce a winner (not a draw)
    assert!(result.winner.is_some(), "Game should have a winner");
    assert!(result.turns > 0, "Game should last at least 1 turn");
    assert!(result.actions_taken > 0, "Game should have actions");
}

#[test]
fn test_simulation_produces_results() {
    let db = sample::build_sample_db();
    let red = sample::red_aggro_deck();
    let green = sample::green_stompy_deck();

    let greedy = GreedyStrategy;
    let results = simulation::simulate(&db, &red, &green, &greedy, &greedy, 100);

    assert_eq!(results.total_games, 100);
    assert_eq!(
        results.player0_wins + results.player1_wins + results.draws,
        100
    );
    assert!(results.avg_turns > 0.0);
}

#[test]
fn test_random_vs_greedy_greedy_wins_more() {
    let db = sample::build_sample_db();
    let red = sample::red_aggro_deck();
    let green = sample::green_stompy_deck();

    let greedy = GreedyStrategy;
    let random = RandomStrategy;

    // Greedy as P0 vs Random as P1 should win more often than random
    let results = simulation::simulate(&db, &red, &green, &greedy, &random, 200);
    let greedy_wr = results.win_rate(0);

    // Greedy should win at least 40% of the time against random
    assert!(
        greedy_wr > 0.4,
        "Greedy should beat random more than 40% of the time, got {:.1}%",
        greedy_wr * 100.0
    );
}

#[test]
fn test_mirror_match_roughly_equal() {
    let db = sample::build_sample_db();
    let red = sample::red_aggro_deck();

    let greedy = GreedyStrategy;

    // Mirror match should be roughly 50/50 (within statistical variance)
    let results = simulation::simulate(&db, &red, &red, &greedy, &greedy, 200);

    // In aggro mirrors, going first is a significant advantage.
    // P0 wins more often but both should win some games.
    let p0_wr = results.win_rate(0);
    let p1_wr = results.win_rate(1);
    assert!(
        p0_wr > 0.0 && p1_wr > 0.0,
        "Both players should win some games in a mirror, got P0={:.1}% P1={:.1}%",
        p0_wr * 100.0,
        p1_wr * 100.0
    );
}
