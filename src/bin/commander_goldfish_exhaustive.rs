//! Commander Goldfish Exhaustive Search
//!
//! Exhaustively explores every legal pilot action branch (no MCCFR) up to turn 20,
//! while treating the opponent as deterministic goldfish.
//!
//! Usage:
//!   cargo run --release --bin commander_goldfish_exhaustive
//!
//! Options (environment variables):
//!   DECK=kinnan        Deck to use: "kinnan" or "brimaz" (default: kinnan)
//!   TURNS=20           Maximum turn horizon for exhaustive search (default: 20)
//!   STATES=1           Number of independently initialized opening states (default: 1)
//!   ACTION_LIMIT=50000 Max actions per line before treated as draw (default: 50000)

use std::collections::HashSet;
use std::sync::atomic::{AtomicBool, AtomicU32, AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Instant;

use mtg_gto::action::{legal_actions, Action};
use mtg_gto::card::sample;
use mtg_gto::game::{GameState, PlayerIndex};
use mtg_gto::rules;
use mtg_gto::strategy::{GoldfishStrategy, Strategy};
use rayon::prelude::*;

#[derive(Debug, Clone)]
struct WinLine {
    turns: u32,
    actions_taken: u32,
    final_life: [i32; 2],
    actions: Vec<Action>,
    state_index: u32,
}

#[derive(Debug, Default, Clone)]
struct ExhaustiveStats {
    nodes_explored: u64,
    terminal_lines: u64,
    wins: u64,
    losses: u64,
    draws: u64,
    loops_cut: u64,
    best_win: Option<WinLine>,
}

fn main() {
    let deck_name = std::env::var("DECK").unwrap_or_else(|_| "kinnan".to_string());
    let max_turns: u32 = std::env::var("TURNS")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(20);
    let num_states: u32 = std::env::var("STATES")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(1);
    let action_limit: u32 = std::env::var("ACTION_LIMIT")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(50_000);

    let db = sample::build_sample_db();
    let (deck, commander, tutor_targets) = match deck_name.as_str() {
        "brimaz" => {
            let (d, c) = sample::brimaz_commander_deck();
            (d, c, Vec::new())
        }
        _ => sample::kinnan_commander_deck(),
    };

    let commander_name = db.get(commander).map(|d| d.name.as_str()).unwrap_or("?");

    println!("Commander Goldfish Exhaustive Search");
    println!("===================================");
    println!("Deck:         {} ({})", deck_name, commander_name);
    println!("Turn horizon: {}", max_turns);
    println!("States:       {}", num_states);
    println!("Action limit: {}", action_limit);
    println!();

    let mut total = ExhaustiveStats::default();
    let progress_nodes = Arc::new(AtomicU64::new(0));
    let progress_states_done = Arc::new(AtomicU32::new(0));
    let progress_done = Arc::new(AtomicBool::new(false));

    let t0 = Instant::now();
    let progress_printer = {
        let nodes = Arc::clone(&progress_nodes);
        let states_done = Arc::clone(&progress_states_done);
        let done = Arc::clone(&progress_done);

        std::thread::spawn(move || loop {
            std::thread::sleep(std::time::Duration::from_secs(2));
            if done.load(Ordering::Relaxed) {
                break;
            }
            let elapsed = t0.elapsed().as_secs_f64();
            let explored = nodes.load(Ordering::Relaxed);
            let completed = states_done.load(Ordering::Relaxed);
            let nps = if elapsed > 0.0 {
                explored as f64 / elapsed
            } else {
                0.0
            };
            eprint!(
                "\rprogress: states {}/{} | nodes {} | {:.0} nodes/s | {:.1}s elapsed   ",
                completed, num_states, explored, nps, elapsed
            );
        })
    };

    let mut per_state_results: Vec<(u32, ExhaustiveStats)> = (1..=num_states)
        .into_par_iter()
        .map(|state_idx| {
            let mut state = GameState::new_commander(2);
            state.card_db = Some(Arc::new(db.clone()));
            rules::setup_commander_game(&mut state, &deck, &deck, commander, commander);
            rules::set_tutor_targets(&mut state, 0, &tutor_targets);
            rules::set_tutor_targets(&mut state, 1, &tutor_targets);

            let mut path = Vec::new();
            let mut seen_hashes = HashSet::new();
            let mut local = ExhaustiveStats::default();
            explore_all_branches(
                state,
                0,
                max_turns,
                action_limit,
                0,
                &mut path,
                &mut seen_hashes,
                &mut local,
                state_idx,
                &progress_nodes,
            );

            progress_states_done.fetch_add(1, Ordering::Relaxed);
            (state_idx, local)
        })
        .collect();

    per_state_results.sort_by_key(|(state_idx, _)| *state_idx);
    for (state_idx, local) in per_state_results {
        println!(
            "State #{}: nodes={} terminals={} W/L/D={}/{}/{} loops_cut={}",
            state_idx,
            local.nodes_explored,
            local.terminal_lines,
            local.wins,
            local.losses,
            local.draws,
            local.loops_cut
        );
        merge_stats(&mut total, local);
    }

    progress_done.store(true, Ordering::Relaxed);
    eprint!(
        "\rprogress: states {}/{} | nodes {} | {:.1}s elapsed                           ",
        progress_states_done.load(Ordering::Relaxed),
        num_states,
        progress_nodes.load(Ordering::Relaxed),
        t0.elapsed().as_secs_f64()
    );
    eprintln!();
    let _ = progress_printer.join();

    println!();
    println!("=== Aggregate ===");
    println!("Nodes explored: {}", total.nodes_explored);
    println!("Terminal lines: {}", total.terminal_lines);
    println!(
        "Wins/Losses/Draws: {}/{}/{}",
        total.wins, total.losses, total.draws
    );
    println!("Loop cuts: {}", total.loops_cut);

    if let Some(win) = &total.best_win {
        println!(
            "Fastest win: state #{} on T{} ({} actions, life {}/{})",
            win.state_index, win.turns, win.actions_taken, win.final_life[0], win.final_life[1]
        );
        for (i, action) in win.actions.iter().enumerate().take(120) {
            println!("  {:>3}. {}", i + 1, action);
        }
        if win.actions.len() > 120 {
            println!("  ... ({} more actions)", win.actions.len() - 120);
        }
    } else {
        println!("Fastest win: none found within exhaustive horizon.");
    }
}

#[allow(clippy::too_many_arguments)]
fn explore_all_branches(
    mut state: GameState,
    pilot: PlayerIndex,
    max_turns: u32,
    action_limit: u32,
    actions_taken: u32,
    path: &mut Vec<Action>,
    seen_hashes: &mut HashSet<u64>,
    stats: &mut ExhaustiveStats,
    state_index: u32,
    progress_nodes: &AtomicU64,
) {
    stats.nodes_explored += 1;
    progress_nodes.fetch_add(1, Ordering::Relaxed);

    if is_terminal_or_limited(&state, max_turns, action_limit, actions_taken) {
        record_terminal(stats, &state, pilot, actions_taken, path, state_index);
        return;
    }

    // Collapse opponent actions deterministically (goldfish).
    let goldfish = GoldfishStrategy;
    while state.priority_player != pilot && !state.game_over {
        if is_terminal_or_limited(
            &state,
            max_turns,
            action_limit,
            actions_taken + path.len() as u32,
        ) {
            record_terminal(stats, &state, pilot, actions_taken, path, state_index);
            return;
        }
        let action = goldfish.choose_action(&state, state.priority_player);
        rules::apply_action(&mut state, &action);
    }

    if is_terminal_or_limited(&state, max_turns, action_limit, actions_taken) {
        record_terminal(stats, &state, pilot, actions_taken, path, state_index);
        return;
    }

    // Cycle cut: if identical state is revisited in this DFS path, this line loops.
    let state_hash = hash_state(&state);
    if !seen_hashes.insert(state_hash) {
        stats.terminal_lines += 1;
        stats.draws += 1;
        stats.loops_cut += 1;
        return;
    }

    let actions = legal_actions(&state);
    if actions.is_empty() {
        let mut next = state;
        let action = Action::PassPriority;
        rules::apply_action(&mut next, &action);
        path.push(action);
        explore_all_branches(
            next,
            pilot,
            max_turns,
            action_limit,
            actions_taken + 1,
            path,
            seen_hashes,
            stats,
            state_index,
            progress_nodes,
        );
        path.pop();
        seen_hashes.remove(&state_hash);
        return;
    }

    for action in actions {
        let mut next = state.clone();
        rules::apply_action(&mut next, &action);
        path.push(action);
        explore_all_branches(
            next,
            pilot,
            max_turns,
            action_limit,
            actions_taken + 1,
            path,
            seen_hashes,
            stats,
            state_index,
            progress_nodes,
        );
        path.pop();
    }

    seen_hashes.remove(&state_hash);
}

fn is_terminal_or_limited(
    state: &GameState,
    max_turns: u32,
    action_limit: u32,
    actions_taken: u32,
) -> bool {
    state.game_over || state.turn_number > max_turns || actions_taken >= action_limit
}

fn record_terminal(
    stats: &mut ExhaustiveStats,
    state: &GameState,
    pilot: PlayerIndex,
    actions_taken: u32,
    path: &[Action],
    state_index: u32,
) {
    stats.terminal_lines += 1;
    match state.winner {
        Some(w) if w == pilot => {
            stats.wins += 1;
            let candidate = WinLine {
                turns: state.turn_number,
                actions_taken,
                final_life: [state.players[0].life, state.players[1].life],
                actions: path.to_vec(),
                state_index,
            };
            let replace = match &stats.best_win {
                None => true,
                Some(existing) => {
                    candidate.turns < existing.turns
                        || (candidate.turns == existing.turns
                            && candidate.actions_taken < existing.actions_taken)
                }
            };
            if replace {
                stats.best_win = Some(candidate);
            }
        }
        Some(_) => stats.losses += 1,
        None => stats.draws += 1,
    }
}

fn hash_state(state: &GameState) -> u64 {
    use std::hash::{Hash, Hasher};

    let bytes = bincode::serialize(state).unwrap_or_default();
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    bytes.hash(&mut hasher);
    hasher.finish()
}

fn merge_stats(total: &mut ExhaustiveStats, local: ExhaustiveStats) {
    total.nodes_explored += local.nodes_explored;
    total.terminal_lines += local.terminal_lines;
    total.wins += local.wins;
    total.losses += local.losses;
    total.draws += local.draws;
    total.loops_cut += local.loops_cut;

    if let Some(candidate) = local.best_win {
        let replace = match &total.best_win {
            None => true,
            Some(existing) => {
                candidate.turns < existing.turns
                    || (candidate.turns == existing.turns
                        && candidate.actions_taken < existing.actions_taken)
            }
        };
        if replace {
            total.best_win = Some(candidate);
        }
    }
}
