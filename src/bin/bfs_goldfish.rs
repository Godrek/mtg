//! BFS Goldfish Solver — Exhaustive DFS with Branch-and-Bound
//!
//! Finds the minimum-turn goldfish kill for the Kinnan cEDH commander deck
//! by exhaustively searching all play lines with aggressive pruning.
//!
//! Usage:
//!   cargo run --release --bin bfs_goldfish
//!
//! Environment variables:
//!   SEEDS=10        Number of random seeds to test (default: 10)
//!   SEED_START=0    First seed value (default: 0)
//!   MAX_TURN=10     Prune branches beyond this turn (default: 10)
//!   TIMEOUT=30      Seconds per seed before aborting (default: 30)
//!   MAX_STATES=500000  Max states explored per seed (default: 500K)
//!   VERBOSE=0       Print action traces for wins (default: 0)
//!   TRACE=0         Print DFS decision trace (default: 0)

use std::collections::{HashMap, HashSet};
use std::hash::{Hash, Hasher};
use std::sync::Arc;
use std::time::Instant;

use mtg_gto::action::{legal_actions, Action};
use mtg_gto::card::sample::{self, ids};
use mtg_gto::card::CardId;
use mtg_gto::combo::ComboCategory;
use mtg_gto::combo_discovery::DiscoveryConfig;
use mtg_gto::game::{GameState, Phase};
use mtg_gto::rules;
use mtg_gto::simulation::format_action_name;
use mtg_gto::strategy::{GreedyStrategy, Strategy};

/// Record of an action taken during search, for trace reconstruction.
#[derive(Clone)]
struct ActionRecord {
    turn: u32,
    #[allow(dead_code)]
    action: Action,
    description: String,
}

/// Statistics for a single seed's search.
struct SearchStats {
    states_explored: u64,
    states_pruned_bound: u64,
    states_pruned_dedup: u64,
    max_depth_reached: usize,
    timed_out: bool,
    hit_state_cap: bool,
}

impl SearchStats {
    fn new() -> Self {
        SearchStats {
            states_explored: 0,
            states_pruned_bound: 0,
            states_pruned_dedup: 0,
            max_depth_reached: 0,
            timed_out: false,
            hit_state_cap: false,
        }
    }
}

/// Resource limits for a single search.
struct SearchLimits {
    deadline: Instant,
    max_states: u64,
    max_visited: usize,
    max_depth: usize,
    trace: bool,
}

/// Dead cards that are useless in goldfish (counterspells, opponent-dependent).
fn dead_card_ids() -> HashSet<CardId> {
    [
        ids::AN_OFFER_YOU_CANT_REFUSE,
        ids::FIERCE_GUARDIANSHIP,
        ids::FLUSTERSTORM,
        ids::FORCE_OF_NEGATION,
        ids::FORCE_OF_WILL,
        ids::MENTAL_MISSTEP,
        ids::MINDBREAK_TRAP,
        ids::SWAN_SONG,
        ids::MUDDLE_THE_MIXTURE,
        ids::CYCLONIC_RIFT,
        ids::INTO_THE_FLOOD_MAW,
        ids::VEIL_OF_SUMMER,
        ids::SINK_INTO_STUPOR,
        ids::MYSTIC_REMORA,
        ids::RHYSTIC_STUDY,
    ]
    .into_iter()
    .collect()
}

/// Tutor targets worth fetching in goldfish — only combo pieces.
const TUTOR_WORTHY: [CardId; 3] = [
    ids::BASALT_MONOLITH,
    ids::WALKING_BALLISTA,
    ids::GRIM_MONOLITH,
];

/// Check if any action is an instant-win combo macro.
fn find_instant_win(state: &GameState, actions: &[Action]) -> Option<Action> {
    let registry = state.combo_registry.as_ref()?;
    for action in actions {
        if let Action::ActivateMacro { combo_id } = action {
            if let Some(combo) = registry.get(*combo_id) {
                if combo.categories.contains(&ComboCategory::InfiniteDamage) {
                    return Some(action.clone());
                }
            }
        }
    }
    None
}

/// Prune actions to reduce branching factor.
fn prune_actions(
    state: &GameState,
    actions: &[Action],
    dead_cards: &HashSet<CardId>,
) -> Vec<Action> {
    // 0. Phase-based auto-pass: during non-strategic phases on our turn, just pass.
    // In goldfish, meaningful decisions happen only during:
    //   - Upkeep (instant-speed tutors before draw)
    //   - PreCombatMain (play lands, cast spells, activate abilities)
    if state.active_player == 0 {
        let dominated_phase = matches!(
            state.phase,
            Phase::Draw
                | Phase::BeginningOfCombat
                | Phase::DeclareBlockers
                | Phase::FirstStrikeDamage
                | Phase::CombatDamage
                | Phase::EndOfCombat
                | Phase::PostCombatMain
                | Phase::EndStep
                | Phase::Cleanup
        );
        if dominated_phase {
            if let Some(win) = find_instant_win(state, actions) {
                return vec![win];
            }
            if actions.iter().any(|a| matches!(a, Action::EndTurn)) {
                return vec![Action::EndTurn];
            }
            return vec![Action::PassPriority];
        }
    }

    // 1. Instant win — always take it.
    if let Some(win) = find_instant_win(state, actions) {
        return vec![win];
    }

    // 2. Forced ordering — no strategic value in goldfish.
    for action in actions {
        if matches!(
            action,
            Action::OrderTriggers { .. } | Action::ChooseReplacementOrder { .. }
        ) {
            return vec![action.clone()];
        }
    }

    // 3. Mulligan/discard — use greedy strategy (single deterministic choice).
    let has_mulligan = actions
        .iter()
        .any(|a| matches!(a, Action::MulliganKeep | Action::MulliganMulligan));
    let all_discard = actions
        .iter()
        .all(|a| matches!(a, Action::Discard { .. }));
    let has_bottom = actions
        .iter()
        .any(|a| matches!(a, Action::MulliganBottomCard { .. }));
    if has_mulligan || all_discard || has_bottom {
        let greedy = GreedyStrategy;
        let choice = greedy.choose_action(state, state.priority_player);
        return vec![choice];
    }

    // 4. Tutor restriction.
    if actions
        .iter()
        .any(|a| matches!(a, Action::ChooseTutorTarget { .. }))
    {
        let is_fetch = state
            .pending_tutor
            .as_ref()
            .map(|pt| !pt.subtype_filter.is_empty())
            .unwrap_or(false);
        if is_fetch {
            // Fetch land — just pick first target (lands are mostly equivalent).
            for a in actions {
                if matches!(a, Action::ChooseTutorTarget { .. }) {
                    return vec![a.clone()];
                }
            }
            return vec![Action::PassPriority];
        }
        // Regular tutor — only combo pieces.
        let worthy: Vec<Action> = actions
            .iter()
            .filter(|a| matches!(a, Action::ChooseTutorTarget { card_id } if TUTOR_WORTHY.contains(card_id)))
            .cloned()
            .collect();
        if worthy.is_empty() {
            return vec![Action::PassPriority];
        }
        return worthy;
    }

    // 5. Combat skip — goldfish doesn't need combat.
    if actions
        .iter()
        .any(|a| matches!(a, Action::DeclareAttackers { .. }))
    {
        return vec![Action::DeclareAttackers {
            attackers: vec![],
        }];
    }
    if actions
        .iter()
        .any(|a| matches!(a, Action::DeclareBlockers { .. }))
    {
        return vec![Action::DeclareBlockers { blocks: vec![] }];
    }

    // 6. Filter dead cards and mana abilities.
    let mut result = Vec::new();
    for action in actions {
        match action {
            Action::CastSpell { object_id, .. } => {
                if let Some(inst) = state.objects.get(object_id) {
                    if dead_cards.contains(&inst.card_def_id) {
                        continue;
                    }
                }
                result.push(action.clone());
            }
            // Engine auto-taps; manual mana abilities waste branching.
            Action::ActivateManaAbility { .. } => continue,
            _ => result.push(action.clone()),
        }
    }

    // 7. Auto-pass if nothing meaningful remains.
    let has_meaningful = result.iter().any(|a| {
        matches!(
            a,
            Action::PlayLand { .. }
                | Action::CastSpell { .. }
                | Action::CastCommander { .. }
                | Action::ActivateAbility { .. }
                | Action::ActivateMacro { .. }
                | Action::Equip { .. }
                | Action::ActivateLoyalty { .. }
                | Action::CastFromGraveyard { .. }
        )
    });
    if !has_meaningful {
        if result.iter().any(|a| matches!(a, Action::EndTurn)) {
            return vec![Action::EndTurn];
        }
        return vec![Action::PassPriority];
    }

    // 8. Reorder: meaningful actions first, Pass/EndTurn last.
    result.sort_by_key(|a| match a {
        Action::ActivateMacro { .. } => 0,
        Action::CastCommander { .. } => 1,
        Action::CastSpell { .. } => 2,
        Action::PlayLand { .. } => 3,
        Action::ActivateAbility { .. } => 4,
        Action::EndTurn => 6,
        Action::PassPriority => 7,
        _ => 5,
    });

    result
}

/// Compute a state fingerprint for deduplication.
fn fingerprint(state: &GameState) -> u64 {
    let mut hasher = std::collections::hash_map::DefaultHasher::new();

    state.turn_number.hash(&mut hasher);
    std::mem::discriminant(&state.phase).hash(&mut hasher);
    state.active_player.hash(&mut hasher);
    state.priority_player.hash(&mut hasher);

    let p0 = &state.players[0];
    p0.life.hash(&mut hasher);
    p0.mana_pool.white.hash(&mut hasher);
    p0.mana_pool.blue.hash(&mut hasher);
    p0.mana_pool.black.hash(&mut hasher);
    p0.mana_pool.red.hash(&mut hasher);
    p0.mana_pool.green.hash(&mut hasher);
    p0.mana_pool.colorless.hash(&mut hasher);
    p0.land_plays_remaining.hash(&mut hasher);
    p0.commander_tax.hash(&mut hasher);

    let mut hand_ids: Vec<CardId> = p0
        .hand
        .iter()
        .filter_map(|oid| state.objects.get(oid).map(|inst| inst.card_def_id))
        .collect();
    hand_ids.sort_unstable();
    hand_ids.hash(&mut hasher);

    let mut bf: Vec<(CardId, bool, usize)> = state
        .battlefield
        .iter()
        .filter_map(|oid| {
            state
                .objects
                .get(oid)
                .map(|inst| (inst.card_def_id, inst.tapped, inst.controller))
        })
        .collect();
    bf.sort_unstable();
    bf.hash(&mut hasher);

    let mut cmd: Vec<CardId> = p0
        .command_zone
        .iter()
        .filter_map(|oid| state.objects.get(oid).map(|inst| inst.card_def_id))
        .collect();
    cmd.sort_unstable();
    cmd.hash(&mut hasher);

    state.stack.len().hash(&mut hasher);
    p0.library.len().hash(&mut hasher);
    state.players[1].life.hash(&mut hasher);

    hasher.finish()
}

/// Core DFS search with branch-and-bound and resource limits.
fn dfs_search(
    state: &mut GameState,
    best_win_turn: &mut u32,
    best_sequence: &mut Vec<ActionRecord>,
    current_sequence: &mut Vec<ActionRecord>,
    dead_cards: &HashSet<CardId>,
    stats: &mut SearchStats,
    visited: &mut HashMap<u64, u32>,
    limits: &SearchLimits,
) {
    // --- Resource checks (every 1024 states to amortize syscall cost) ---
    if stats.states_explored & 0x3FF == 0 && stats.states_explored > 0 {
        if Instant::now() >= limits.deadline {
            stats.timed_out = true;
            return;
        }
    }
    if stats.timed_out || stats.hit_state_cap {
        return;
    }
    if stats.states_explored >= limits.max_states {
        stats.hit_state_cap = true;
        return;
    }

    if current_sequence.len() > stats.max_depth_reached {
        stats.max_depth_reached = current_sequence.len();
    }

    // Win check
    if state.game_over {
        if state.winner == Some(0) && state.turn_number < *best_win_turn {
            *best_win_turn = state.turn_number;
            *best_sequence = current_sequence.clone();
        }
        if limits.trace {
            eprintln!(
                "  [TERM] game_over, winner={:?}, turn={}",
                state.winner, state.turn_number
            );
        }
        return;
    }

    // Bound prune
    if state.turn_number >= *best_win_turn {
        stats.states_pruned_bound += 1;
        return;
    }

    // Depth limit
    if current_sequence.len() > limits.max_depth {
        return;
    }

    // State deduplication (with cap on HashMap size)
    let fp = fingerprint(state);
    if let Some(&prev_turn) = visited.get(&fp) {
        if prev_turn <= state.turn_number {
            stats.states_pruned_dedup += 1;
            return;
        }
    }
    if visited.len() < limits.max_visited {
        visited.insert(fp, state.turn_number);
    }

    stats.states_explored += 1;

    if limits.trace && stats.states_explored <= 100 {
        eprintln!(
            "  [STATE #{}] T{} {:?} prio={} depth={} hand={} bf={} stack={}",
            stats.states_explored,
            state.turn_number,
            state.phase,
            state.priority_player,
            current_sequence.len(),
            state.players[0].hand.len(),
            state.battlefield.len(),
            state.stack.len(),
        );
    }

    // Opponent's turn — fast-forward entirely.
    if state.active_player != 0 {
        let mut clone = state.clone();
        rules::fast_forward_goldfish_turn(&mut clone);
        dfs_search(
            &mut clone,
            best_win_turn,
            best_sequence,
            current_sequence,
            dead_cards,
            stats,
            visited,
            limits,
        );
        return;
    }

    // During our turn, auto-pass when opponent has priority
    if state.priority_player != 0 {
        let mut clone = state.clone();
        rules::apply_action(&mut clone, &Action::PassPriority);
        dfs_search(
            &mut clone,
            best_win_turn,
            best_sequence,
            current_sequence,
            dead_cards,
            stats,
            visited,
            limits,
        );
        return;
    }

    // Get and prune legal actions
    let actions = legal_actions(state);
    if actions.is_empty() {
        return;
    }
    let pruned = prune_actions(state, &actions, dead_cards);
    if pruned.is_empty() {
        return;
    }

    if limits.trace && stats.states_explored <= 100 {
        eprintln!(
            "    legal={} pruned={} actions: {}",
            actions.len(),
            pruned.len(),
            pruned
                .iter()
                .map(|a| format_action_name(state, a))
                .collect::<Vec<_>>()
                .join(", ")
        );
    }

    // Apply each pruned action (single-action optimization: no branching overhead)
    if pruned.len() == 1 {
        let action = pruned[0].clone();
        let desc = format_action_name(state, &action);
        let mut clone = state.clone();
        let turn = clone.turn_number;
        rules::apply_action(&mut clone, &action);
        current_sequence.push(ActionRecord {
            turn,
            action,
            description: desc,
        });
        dfs_search(
            &mut clone,
            best_win_turn,
            best_sequence,
            current_sequence,
            dead_cards,
            stats,
            visited,
            limits,
        );
        current_sequence.pop();
        return;
    }

    for action in &pruned {
        // Bail early if we hit limits
        if stats.timed_out || stats.hit_state_cap {
            return;
        }
        let desc = format_action_name(state, action);
        let mut clone = state.clone();
        let turn = clone.turn_number;
        rules::apply_action(&mut clone, action);
        current_sequence.push(ActionRecord {
            turn,
            action: action.clone(),
            description: desc,
        });
        dfs_search(
            &mut clone,
            best_win_turn,
            best_sequence,
            current_sequence,
            dead_cards,
            stats,
            visited,
            limits,
        );
        current_sequence.pop();
    }
}

/// Run the solver for a single seed.
fn solve_seed(
    db: &mtg_gto::game::CardDatabase,
    deck: &[CardId],
    commander: CardId,
    tutor_targets: &[CardId],
    seed: u64,
    max_turn: u32,
    timeout_secs: u64,
    max_states: u64,
    verbose: bool,
    trace: bool,
) -> (Option<u32>, SearchStats, Vec<ActionRecord>) {
    let arc_db = Arc::new(db.clone());

    let mut state = GameState::new_commander(2);
    state.card_db = Some(arc_db);
    rules::setup_commander_game_seeded(&mut state, deck, deck, commander, commander, seed);
    rules::set_tutor_targets(&mut state, 0, tutor_targets);

    // Discover combos
    let mut combo_cards = vec![commander];
    combo_cards.extend_from_slice(tutor_targets);
    let (mut registry, _) = mtg_gto::combo_discovery::discover_and_register(
        db,
        &combo_cards,
        &DiscoveryConfig::default(),
    );
    mtg_gto::combo::register_ballista_win_combo(&mut registry);
    state.combo_registry = Some(Arc::new(registry));

    let dead_cards = dead_card_ids();
    let mut best_win_turn = max_turn + 1;
    let mut best_sequence = Vec::new();
    let mut current_sequence = Vec::new();
    let mut stats = SearchStats::new();
    // Pre-allocate visited with a reasonable capacity, capped to prevent OOM.
    let visited_cap = (max_states as usize).min(1_000_000);
    let mut visited = HashMap::with_capacity(visited_cap.min(100_000));

    let limits = SearchLimits {
        deadline: Instant::now() + std::time::Duration::from_secs(timeout_secs),
        max_states,
        max_visited: visited_cap,
        max_depth: 500,
        trace,
    };

    dfs_search(
        &mut state,
        &mut best_win_turn,
        &mut best_sequence,
        &mut current_sequence,
        &dead_cards,
        &mut stats,
        &mut visited,
        &limits,
    );

    let win_turn = if best_win_turn <= max_turn {
        Some(best_win_turn)
    } else {
        None
    };

    if verbose {
        if let Some(t) = win_turn {
            println!("  Winning line (T{}):", t);
            let mut last_turn = 0;
            for record in &best_sequence {
                if record.turn != last_turn {
                    last_turn = record.turn;
                    println!("  --- Turn {} ---", last_turn);
                }
                println!("    {}", record.description);
            }
        }
    }

    (win_turn, stats, best_sequence)
}

fn main() {
    let num_seeds: u64 = std::env::var("SEEDS")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(10);
    let seed_start: u64 = std::env::var("SEED_START")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(0);
    let max_turn: u32 = std::env::var("MAX_TURN")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(10);
    let timeout_secs: u64 = std::env::var("TIMEOUT")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(30);
    let max_states: u64 = std::env::var("MAX_STATES")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(500_000);
    let verbose: bool = std::env::var("VERBOSE")
        .ok()
        .and_then(|v| v.parse::<u32>().ok())
        .unwrap_or(0)
        > 0;
    let trace: bool = std::env::var("TRACE")
        .ok()
        .and_then(|v| v.parse::<u32>().ok())
        .unwrap_or(0)
        > 0;

    let db = sample::build_sample_db();
    let (deck, commander, tutor_targets) = sample::kinnan_commander_deck();
    let commander_name = db.get(commander).map(|d| d.name.as_str()).unwrap_or("?");

    println!("BFS Goldfish Solver");
    println!("===================");
    println!("Commander:  {}", commander_name);
    println!("Seeds:      {}-{}", seed_start, seed_start + num_seeds - 1);
    println!("Max turn:   {}", max_turn);
    println!("Timeout:    {}s per seed", timeout_secs);
    println!("Max states: {}", max_states);
    println!("Verbose:    {}", verbose);
    println!();

    let total_start = Instant::now();

    let mut kill_turns: HashMap<u32, u32> = HashMap::new();
    let mut wins = 0u32;
    let mut total_states = 0u64;
    let mut timeouts = 0u32;

    for seed in seed_start..(seed_start + num_seeds) {
        let start = Instant::now();
        let (win_turn, stats, _sequence) = solve_seed(
            &db,
            &deck,
            commander,
            &tutor_targets,
            seed,
            max_turn,
            timeout_secs,
            max_states,
            verbose,
            trace,
        );
        let elapsed = start.elapsed();

        let suffix = if stats.timed_out {
            timeouts += 1;
            " [TIMEOUT]"
        } else if stats.hit_state_cap {
            timeouts += 1;
            " [STATE CAP]"
        } else {
            ""
        };

        match win_turn {
            Some(t) => {
                println!(
                    "Seed {:3}: Win T{} ({} states, {:.1}s){}",
                    seed, t, stats.states_explored, elapsed.as_secs_f64(), suffix
                );
                *kill_turns.entry(t).or_insert(0) += 1;
                wins += 1;
            }
            None => {
                println!(
                    "Seed {:3}: No win by T{} ({} states, {:.1}s){}",
                    seed, max_turn, stats.states_explored, elapsed.as_secs_f64(), suffix
                );
            }
        }
        total_states += stats.states_explored;
    }

    let total_elapsed = total_start.elapsed();

    println!();
    println!("Summary");
    println!("=======");
    println!(
        "Win rate:   {}/{} ({:.1}%)",
        wins,
        num_seeds,
        100.0 * wins as f64 / num_seeds as f64
    );
    if timeouts > 0 {
        println!("Timeouts:   {}", timeouts);
    }
    println!("Total time: {:.1}s", total_elapsed.as_secs_f64());
    println!(
        "Avg states: {:.0}",
        total_states as f64 / num_seeds as f64
    );

    if !kill_turns.is_empty() {
        println!();
        println!("Kill turn distribution:");
        let mut turns: Vec<u32> = kill_turns.keys().cloned().collect();
        turns.sort();
        for t in turns {
            println!("  T{}: {} games", t, kill_turns[&t]);
        }
    }
}
