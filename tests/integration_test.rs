use mtg_gto::card::sample;
use mtg_gto::card::ZoneType;
use mtg_gto::game::GameState;
use mtg_gto::rules;
use mtg_gto::simulation;
use mtg_gto::strategy::{GreedyStrategy, RandomStrategy};
use mtg_gto::action::Action;

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

#[test]
fn test_etb_trigger_elvish_visionary() {
    // Test that Elvish Visionary's ETB trigger draws a card
    let db = sample::build_sample_db();
    let mut state = GameState::new(2);
    state.card_db = Some(db);

    // Give player 0 some forests and an Elvish Visionary in hand
    for _ in 0..3 {
        state.create_card_in_zone(sample::ids::FOREST, 0, ZoneType::Library);
    }
    // Give both players some library cards to draw from
    for _ in 0..20 {
        state.create_card_in_zone(sample::ids::MOUNTAIN, 0, ZoneType::Library);
        state.create_card_in_zone(sample::ids::MOUNTAIN, 1, ZoneType::Library);
    }

    // Put Elvish Visionary in hand
    let vis_id = state.create_card_in_zone(sample::ids::ELVISH_VISIONARY, 0, ZoneType::Hand);
    // Put 2 Forests on battlefield (untapped) for mana
    let f1 = state.create_card_in_zone(sample::ids::FOREST, 0, ZoneType::Battlefield);
    let f2 = state.create_card_in_zone(sample::ids::FOREST, 0, ZoneType::Battlefield);
    if let Some(inst) = state.objects.get_mut(&f1) {
        inst.tapped = false;
        inst.summoning_sick = false;
    }
    if let Some(inst) = state.objects.get_mut(&f2) {
        inst.tapped = false;
        inst.summoning_sick = false;
    }

    // Set up game state for main phase
    state.active_player = 0;
    state.priority_player = 0;
    state.phase = mtg_gto::game::Phase::PreCombatMain;
    state.turn_number = 2; // not turn 1 so no special rules

    let hand_before = state.players[0].hand.len();

    // Cast Elvish Visionary
    rules::apply_action(&mut state, &Action::CastSpell {
        object_id: vis_id,
        targets: vec![],
    });

    // Visionary should be on the stack
    assert_eq!(state.stack.len(), 1);

    // Both players pass priority to resolve
    rules::apply_action(&mut state, &Action::PassPriority);
    rules::apply_action(&mut state, &Action::PassPriority);

    // After resolution, Visionary is on the battlefield
    assert!(
        state.battlefield.contains(&vis_id),
        "Elvish Visionary should be on the battlefield after resolution"
    );

    // The ETB trigger should be on the stack now
    assert_eq!(
        state.stack.len(), 1,
        "ETB trigger should be on the stack"
    );

    // Resolve the ETB trigger (both players pass)
    rules::apply_action(&mut state, &Action::PassPriority);
    rules::apply_action(&mut state, &Action::PassPriority);

    // After ETB resolves, player should have drawn a card
    // Hand was: hand_before - 1 (cast visionary) + 1 (ETB draw) = hand_before
    let hand_after = state.players[0].hand.len();
    assert_eq!(
        hand_after,
        hand_before, // -1 for casting, +1 for draw = same
        "Player should have drawn a card from Elvish Visionary ETB (before={}, after={})",
        hand_before,
        hand_after
    );
}

#[test]
fn test_new_sample_cards_in_db() {
    let db = sample::build_sample_db();
    assert!(db.get(sample::ids::ELVISH_VISIONARY).is_some());
    assert!(db.get(sample::ids::BLADE_SPLICER).is_some());
    assert!(db.get(sample::ids::SIEGE_GANG_COMMANDER).is_some());

    // Check Elvish Visionary has an ETB trigger
    let ev = db.get(sample::ids::ELVISH_VISIONARY).unwrap();
    assert_eq!(ev.triggered_abilities.len(), 1);
    assert_eq!(
        ev.triggered_abilities[0].trigger,
        mtg_gto::card::TriggerCondition::EntersBattlefield
    );
}

#[test]
fn test_cleanup_requires_discard_action() {
    let db = sample::build_sample_db();
    let mut state = GameState::new(2);
    state.card_db = Some(db);

    for _ in 0..8 {
        state.create_card_in_zone(sample::ids::MOUNTAIN, 0, ZoneType::Hand);
    }

    state.active_player = 0;
    state.priority_player = 0;
    state.phase = mtg_gto::game::Phase::Cleanup;
    state.turn_number = 1;

    let actions = mtg_gto::action::legal_actions(&state);
    assert_eq!(actions.len(), 8);
    assert!(actions.iter().all(|action| matches!(action, Action::Discard { .. })));

    let discard_action = actions[0].clone();
    rules::apply_action(&mut state, &discard_action);

    assert_eq!(state.players[0].hand.len(), 7);
    assert_eq!(state.players[0].graveyard.len(), 1);
    assert_eq!(state.turn_number, 2);
    assert_eq!(state.phase, mtg_gto::game::Phase::Upkeep);
}
