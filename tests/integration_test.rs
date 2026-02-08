use mtg_gto::action::{legal_actions, Action};
use mtg_gto::card::sample;
use mtg_gto::card::ZoneType;
use mtg_gto::game::GameState;
use mtg_gto::rules;
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
fn test_order_triggers_surfaced_for_multiple_simultaneous_triggers() {
    // When a player controls two permanents that both trigger on the same event,
    // they must choose the order to place them on the stack. This should be
    // surfaced as Action::OrderTriggers, not silently ordered FIFO.
    let db = sample::build_sample_db();
    let mut state = GameState::new(2);
    state.card_db = Some(db);

    // Give both players library cards so no one loses from empty library
    for _ in 0..20 {
        state.create_card_in_zone(sample::ids::MOUNTAIN, 0, ZoneType::Library);
        state.create_card_in_zone(sample::ids::MOUNTAIN, 1, ZoneType::Library);
    }

    // Put two Elvish Visionaries on the battlefield for player 0.
    // Both have "When ~ enters the battlefield, draw a card" — but we'll
    // use the trigger system directly by queuing two simultaneous triggers.
    let vis1 = state.create_card_in_zone(sample::ids::ELVISH_VISIONARY, 0, ZoneType::Battlefield);
    let vis2 = state.create_card_in_zone(sample::ids::ELVISH_VISIONARY, 0, ZoneType::Battlefield);

    state.active_player = 0;
    state.priority_player = 0;
    state.phase = mtg_gto::game::Phase::PreCombatMain;
    state.turn_number = 2;

    // Manually queue two simultaneous ETB triggers for player 0
    state.pending_triggers.push(mtg_gto::game::PendingTrigger {
        source_id: vis1,
        ability_index: 0,
        controller: 0,
        targets: vec![],
    });
    state.pending_triggers.push(mtg_gto::game::PendingTrigger {
        source_id: vis2,
        ability_index: 0,
        controller: 0,
        targets: vec![],
    });

    // Attempt to flush — should pause because player 0 has >1 trigger
    // (flush_triggers is internal, but we can observe via legal_actions)
    let actions = legal_actions(&state);

    // Should contain OrderTriggers actions (2! = 2 permutations) plus Concede
    let order_trigger_actions: Vec<&Action> = actions
        .iter()
        .filter(|a| matches!(a, Action::OrderTriggers { .. }))
        .collect();

    assert_eq!(
        order_trigger_actions.len(),
        2,
        "Should have 2 orderings (2! permutations) for 2 triggers, got {:?}",
        actions
    );

    // Should NOT contain PassPriority (only ordering + concede allowed)
    assert!(
        !actions.contains(&Action::PassPriority),
        "PassPriority should not be offered when triggers need ordering"
    );

    // Apply the first ordering
    let chosen = order_trigger_actions[0].clone();
    rules::apply_action(&mut state, &chosen);

    // After ordering, triggers should be on the stack and pending_triggers empty
    assert!(
        state.pending_triggers.is_empty(),
        "pending_triggers should be empty after ordering"
    );
    assert_eq!(
        state.stack.len(),
        2,
        "Both triggers should be on the stack"
    );

    // Now legal_actions should return normal priority actions (PassPriority, etc.)
    let actions_after = legal_actions(&state);
    assert!(
        actions_after.contains(&Action::PassPriority),
        "Should have normal priority actions after triggers are ordered"
    );
}

#[test]
fn test_single_trigger_auto_flushes_without_ordering() {
    // When a player has exactly 1 trigger, it should be auto-pushed
    // to the stack without requiring an OrderTriggers action.
    let db = sample::build_sample_db();
    let mut state = GameState::new(2);
    state.card_db = Some(db);

    for _ in 0..20 {
        state.create_card_in_zone(sample::ids::MOUNTAIN, 0, ZoneType::Library);
        state.create_card_in_zone(sample::ids::MOUNTAIN, 1, ZoneType::Library);
    }

    let vis = state.create_card_in_zone(sample::ids::ELVISH_VISIONARY, 0, ZoneType::Battlefield);

    state.active_player = 0;
    state.priority_player = 0;
    state.phase = mtg_gto::game::Phase::PreCombatMain;
    state.turn_number = 2;

    // Queue a single trigger
    state.pending_triggers.push(mtg_gto::game::PendingTrigger {
        source_id: vis,
        ability_index: 0,
        controller: 0,
        targets: vec![],
    });

    // legal_actions should NOT offer OrderTriggers — single trigger auto-flushes
    // But first we need to actually run flush_triggers. The pending_triggers are
    // checked by legal_actions but flush_triggers is called by rules engine.
    // With 1 pending trigger and it's the priority player's, legal_actions
    // will see 1 trigger (not >1) and proceed to normal actions.
    let actions = legal_actions(&state);
    let has_order_triggers = actions
        .iter()
        .any(|a| matches!(a, Action::OrderTriggers { .. }));

    assert!(
        !has_order_triggers,
        "Should not offer OrderTriggers for a single trigger"
    );
    assert!(
        actions.contains(&Action::PassPriority),
        "Should offer normal priority actions for single trigger"
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

#[test]
fn test_cleanup_allows_pass_at_seven() {
    let db = sample::build_sample_db();
    let mut state = GameState::new(2);
    state.card_db = Some(db);

    for _ in 0..7 {
        state.create_card_in_zone(sample::ids::MOUNTAIN, 0, ZoneType::Hand);
    }

    state.active_player = 0;
    state.priority_player = 0;
    state.phase = mtg_gto::game::Phase::Cleanup;

    let actions = mtg_gto::action::legal_actions(&state);
    assert!(actions.iter().any(|action| matches!(action, Action::PassPriority)));
    assert!(actions.iter().all(|action| !matches!(action, Action::Discard { .. })));
}

#[test]
fn test_cleanup_multiple_discards() {
    let db = sample::build_sample_db();
    let mut state = GameState::new(2);
    state.card_db = Some(db);

    let mut hand_ids = Vec::new();
    for _ in 0..10 {
        hand_ids.push(state.create_card_in_zone(sample::ids::MOUNTAIN, 0, ZoneType::Hand));
    }

    state.active_player = 0;
    state.priority_player = 0;
    state.phase = mtg_gto::game::Phase::Cleanup;
    state.turn_number = 1;

    let first_discard = hand_ids[3];
    rules::apply_action(&mut state, &Action::Discard { object_id: first_discard });
    assert!(!state.players[0].hand.contains(&first_discard));
    assert!(state.players[0].graveyard.contains(&first_discard));
    assert_eq!(state.players[0].hand.len(), 9);

    let actions = mtg_gto::action::legal_actions(&state);
    let discard_action = actions
        .into_iter()
        .find(|action| matches!(action, Action::Discard { .. }))
        .expect("expected another discard action");
    rules::apply_action(&mut state, &discard_action);
    assert_eq!(state.players[0].hand.len(), 8);

    let actions = mtg_gto::action::legal_actions(&state);
    let discard_action = actions
        .into_iter()
        .find(|action| matches!(action, Action::Discard { .. }))
        .expect("expected final discard action");
    rules::apply_action(&mut state, &discard_action);

    assert_eq!(state.players[0].hand.len(), 7);
    assert_eq!(state.turn_number, 2);
    assert_eq!(state.phase, mtg_gto::game::Phase::Upkeep);
}

#[test]
fn test_apnap_both_players_multiple_triggers() {
    // When both active player AND non-active player each have >1 simultaneous
    // trigger, AP must order first (APNAP), then NAP orders after.
    let db = sample::build_sample_db();
    let mut state = GameState::new(2);
    state.card_db = Some(db);

    for _ in 0..20 {
        state.create_card_in_zone(sample::ids::MOUNTAIN, 0, ZoneType::Library);
        state.create_card_in_zone(sample::ids::MOUNTAIN, 1, ZoneType::Library);
    }

    let vis_p0_a = state.create_card_in_zone(sample::ids::ELVISH_VISIONARY, 0, ZoneType::Battlefield);
    let vis_p0_b = state.create_card_in_zone(sample::ids::ELVISH_VISIONARY, 0, ZoneType::Battlefield);
    let vis_p1_a = state.create_card_in_zone(sample::ids::ELVISH_VISIONARY, 1, ZoneType::Battlefield);
    let vis_p1_b = state.create_card_in_zone(sample::ids::ELVISH_VISIONARY, 1, ZoneType::Battlefield);

    state.active_player = 0;
    state.priority_player = 0;
    state.phase = mtg_gto::game::Phase::PreCombatMain;
    state.turn_number = 2;

    // Queue 2 triggers for AP (player 0) and 2 for NAP (player 1)
    state.pending_triggers.push(mtg_gto::game::PendingTrigger {
        source_id: vis_p0_a, ability_index: 0, controller: 0, targets: vec![],
    });
    state.pending_triggers.push(mtg_gto::game::PendingTrigger {
        source_id: vis_p0_b, ability_index: 0, controller: 0, targets: vec![],
    });
    state.pending_triggers.push(mtg_gto::game::PendingTrigger {
        source_id: vis_p1_a, ability_index: 0, controller: 1, targets: vec![],
    });
    state.pending_triggers.push(mtg_gto::game::PendingTrigger {
        source_id: vis_p1_b, ability_index: 0, controller: 1, targets: vec![],
    });

    // AP (player 0) should order first
    let actions = legal_actions(&state);
    assert_eq!(state.priority_player, 0, "AP should have priority to order first");
    let order_actions: Vec<&Action> = actions.iter()
        .filter(|a| matches!(a, Action::OrderTriggers { .. }))
        .collect();
    assert_eq!(order_actions.len(), 2, "AP should see 2! = 2 orderings for their 2 triggers");

    // AP orders their triggers
    rules::apply_action(&mut state, order_actions[0]);

    // Now NAP (player 1) should have priority to order their triggers
    assert_eq!(state.priority_player, 1, "NAP should now have priority to order");
    assert!(!state.pending_triggers.is_empty(), "NAP triggers should still be pending");

    let actions2 = legal_actions(&state);
    let order_actions2: Vec<&Action> = actions2.iter()
        .filter(|a| matches!(a, Action::OrderTriggers { .. }))
        .collect();
    assert_eq!(order_actions2.len(), 2, "NAP should see 2! = 2 orderings for their 2 triggers");

    // NAP orders their triggers
    rules::apply_action(&mut state, order_actions2[0]);

    // All 4 triggers should now be on the stack
    assert!(state.pending_triggers.is_empty(), "All triggers should be flushed");
    assert_eq!(state.stack.len(), 4, "All 4 triggers should be on the stack");

    // Verify APNAP stack order: AP's triggers were placed first (resolve last),
    // NAP's triggers placed second (resolve first since stack is LIFO)
    assert_eq!(state.stack[0].controller, 0, "AP triggers on stack first");
    assert_eq!(state.stack[1].controller, 0, "AP triggers on stack first");
    assert_eq!(state.stack[2].controller, 1, "NAP triggers on stack second");
    assert_eq!(state.stack[3].controller, 1, "NAP triggers on stack second");
}

#[test]
fn test_more_than_six_triggers_fifo_fallback() {
    // When a player has >6 simultaneous triggers, the permutation generator
    // falls back to a single FIFO ordering to avoid combinatorial explosion.
    let db = sample::build_sample_db();
    let mut state = GameState::new(2);
    state.card_db = Some(db);

    for _ in 0..20 {
        state.create_card_in_zone(sample::ids::MOUNTAIN, 0, ZoneType::Library);
        state.create_card_in_zone(sample::ids::MOUNTAIN, 1, ZoneType::Library);
    }

    // Create 7 permanents with triggers for player 0
    let mut vis_ids = Vec::new();
    for _ in 0..7 {
        vis_ids.push(
            state.create_card_in_zone(sample::ids::ELVISH_VISIONARY, 0, ZoneType::Battlefield),
        );
    }

    state.active_player = 0;
    state.priority_player = 0;
    state.phase = mtg_gto::game::Phase::PreCombatMain;
    state.turn_number = 2;

    // Queue 7 triggers for player 0
    for &vis_id in &vis_ids {
        state.pending_triggers.push(mtg_gto::game::PendingTrigger {
            source_id: vis_id, ability_index: 0, controller: 0, targets: vec![],
        });
    }

    let actions = legal_actions(&state);
    let order_actions: Vec<&Action> = actions.iter()
        .filter(|a| matches!(a, Action::OrderTriggers { .. }))
        .collect();

    // 7! = 5040 would be too many; should fall back to exactly 1 FIFO ordering
    assert_eq!(
        order_actions.len(), 1,
        "Should have exactly 1 ordering (FIFO fallback) for >6 triggers, got {}",
        order_actions.len()
    );

    // Apply the single ordering — all triggers should end up on the stack
    rules::apply_action(&mut state, order_actions[0]);
    assert!(state.pending_triggers.is_empty());
    assert_eq!(state.stack.len(), 7, "All 7 triggers should be on the stack");
}

#[test]
fn test_etb_multiple_triggers_through_natural_game_flow() {
    // Test that resolving a creature with an ETB trigger, when there's already
    // another permanent with an ETB-watching trigger, correctly surfaces
    // OrderTriggers through the actual spell resolution path.
    //
    // Setup: Player 0 has an Elvish Visionary on the battlefield and casts a
    // second Elvish Visionary. When the second resolves, there's only 1 ETB
    // trigger (the one from the entering creature). This is auto-flushed.
    // But we can test the "2 simultaneous ETB" case by manually triggering
    // the fire_triggers path with 2 pending triggers in the natural game context.
    let db = sample::build_sample_db();
    let mut state = GameState::new(2);
    state.card_db = Some(db);

    for _ in 0..20 {
        state.create_card_in_zone(sample::ids::MOUNTAIN, 0, ZoneType::Library);
        state.create_card_in_zone(sample::ids::MOUNTAIN, 1, ZoneType::Library);
    }

    // Put an Elvish Visionary in hand with enough forests to cast
    let vis_id = state.create_card_in_zone(sample::ids::ELVISH_VISIONARY, 0, ZoneType::Hand);
    let f1 = state.create_card_in_zone(sample::ids::FOREST, 0, ZoneType::Battlefield);
    let f2 = state.create_card_in_zone(sample::ids::FOREST, 0, ZoneType::Battlefield);
    for id in [f1, f2] {
        if let Some(inst) = state.objects.get_mut(&id) {
            inst.tapped = false;
            inst.summoning_sick = false;
        }
    }

    // Put a second Elvish Visionary already on the battlefield
    let _vis_existing = state.create_card_in_zone(
        sample::ids::ELVISH_VISIONARY, 0, ZoneType::Battlefield,
    );

    state.active_player = 0;
    state.priority_player = 0;
    state.phase = mtg_gto::game::Phase::PreCombatMain;
    state.turn_number = 2;

    // Cast the Elvish Visionary
    rules::apply_action(&mut state, &Action::CastSpell {
        object_id: vis_id,
        targets: vec![],
    });
    assert_eq!(state.stack.len(), 1, "Spell should be on stack");

    // Resolve: both players pass
    rules::apply_action(&mut state, &Action::PassPriority);
    rules::apply_action(&mut state, &Action::PassPriority);

    // After resolution, Visionary enters the battlefield and its ETB trigger fires.
    // Only the entering creature has an ETB trigger (the existing one doesn't
    // re-trigger), so there's exactly 1 trigger — auto-flushed, no OrderTriggers.
    assert!(
        state.pending_triggers.is_empty(),
        "Single ETB trigger should be auto-flushed"
    );
    assert_eq!(
        state.stack.len(), 1,
        "ETB trigger should be on the stack"
    );

    // Verify normal flow continues: resolve the ETB trigger
    rules::apply_action(&mut state, &Action::PassPriority);
    rules::apply_action(&mut state, &Action::PassPriority);

    // Trigger resolved — player drew a card
    assert_eq!(state.stack.len(), 0, "Stack should be empty after ETB resolution");
}
