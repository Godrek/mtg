use rayon::prelude::*;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

use crate::action::legal_actions;
use crate::card::CardId;
use crate::game::{CardDatabase, GameState, PlayerIndex};
use crate::rules;
use crate::strategy::{GoldfishStrategy, Strategy};

/// Maximum turns before a game is declared a draw.
const MAX_TURNS: u32 = 200;

/// Maximum actions per game before forced draw (prevents infinite loops).
const MAX_ACTIONS: u32 = 50_000;

/// Result of a single simulated game.
#[derive(Debug, Clone)]
pub struct GameResult {
    pub winner: Option<PlayerIndex>,
    pub turns: u32,
    pub actions_taken: u32,
    pub final_life: [i32; 2],
}

/// Aggregate results from many simulated games.
#[derive(Debug, Clone)]
pub struct SimulationResults {
    pub total_games: u64,
    pub player0_wins: u64,
    pub player1_wins: u64,
    pub draws: u64,
    pub avg_turns: f64,
    pub avg_actions: f64,
}

impl SimulationResults {
    pub fn win_rate(&self, player: PlayerIndex) -> f64 {
        let wins = if player == 0 {
            self.player0_wins
        } else {
            self.player1_wins
        };
        wins as f64 / self.total_games as f64
    }

    pub fn display(&self) {
        println!("=== Simulation Results ===");
        println!("Total games: {}", self.total_games);
        println!(
            "Player 0 wins: {} ({:.1}%)",
            self.player0_wins,
            self.win_rate(0) * 100.0
        );
        println!(
            "Player 1 wins: {} ({:.1}%)",
            self.player1_wins,
            self.win_rate(1) * 100.0
        );
        println!("Draws: {}", self.draws);
        println!("Avg turns: {:.1}", self.avg_turns);
        println!("Avg actions: {:.1}", self.avg_actions);
    }
}

/// Run a single game to completion with the given strategies.
pub fn run_game(
    card_db: &CardDatabase,
    deck0: &[CardId],
    deck1: &[CardId],
    strategy0: &dyn Strategy,
    strategy1: &dyn Strategy,
) -> GameResult {
    let db = Arc::new(card_db.clone());
    run_game_inner(db, deck0, deck1, strategy0, strategy1, false)
}

/// Run a single game with optional verbose tracing.
pub fn run_game_verbose(
    card_db: &CardDatabase,
    deck0: &[CardId],
    deck1: &[CardId],
    strategy0: &dyn Strategy,
    strategy1: &dyn Strategy,
) -> GameResult {
    let db = Arc::new(card_db.clone());
    run_game_inner(db, deck0, deck1, strategy0, strategy1, true)
}

fn run_game_inner(
    card_db: Arc<CardDatabase>,
    deck0: &[CardId],
    deck1: &[CardId],
    strategy0: &dyn Strategy,
    strategy1: &dyn Strategy,
    verbose: bool,
) -> GameResult {
    let mut state = GameState::new(2);
    state.card_db = Some(card_db);
    rules::setup_game(&mut state, deck0, deck1);
    run_game_loop(&mut state, strategy0, strategy1, verbose)
}

/// Run a single Commander game to completion with the given strategies.
pub fn run_commander_game(
    card_db: &CardDatabase,
    deck0: &[CardId],
    deck1: &[CardId],
    commander0: CardId,
    commander1: CardId,
    strategy0: &dyn Strategy,
    strategy1: &dyn Strategy,
) -> GameResult {
    let db = Arc::new(card_db.clone());
    run_commander_game_inner(
        db, deck0, deck1, commander0, commander1, strategy0, strategy1, false,
    )
}

fn run_commander_game_inner(
    card_db: Arc<CardDatabase>,
    deck0: &[CardId],
    deck1: &[CardId],
    commander0: CardId,
    commander1: CardId,
    strategy0: &dyn Strategy,
    strategy1: &dyn Strategy,
    verbose: bool,
) -> GameResult {
    let mut state = GameState::new_commander(2);
    state.card_db = Some(card_db);
    rules::setup_commander_game(&mut state, deck0, deck1, commander0, commander1);
    run_game_loop(&mut state, strategy0, strategy1, verbose)
}

/// Shared game loop for both standard and commander formats.
fn run_game_loop(
    state: &mut GameState,
    strategy0: &dyn Strategy,
    strategy1: &dyn Strategy,
    verbose: bool,
) -> GameResult {
    let mut actions_taken: u32 = 0;

    while !state.game_over && state.turn_number <= MAX_TURNS && actions_taken < MAX_ACTIONS {
        let player = state.priority_player;
        let actions = legal_actions(state);

        if actions.is_empty()
            || (actions.len() == 1 && actions[0] == crate::action::Action::PassPriority)
        {
            rules::apply_action(state, &crate::action::Action::PassPriority);
            actions_taken += 1;
            continue;
        }

        let strategy: &dyn Strategy = if player == 0 { strategy0 } else { strategy1 };
        let action = strategy.choose_action(state, player);

        if verbose && actions_taken < 200 {
            let db = state.card_db();
            let action_name = match &action {
                crate::action::Action::CastSpell { object_id, .. }
                | crate::action::Action::CastCommander { object_id, .. } => {
                    let inst = &state.objects[object_id];
                    format!(
                        "Cast {}",
                        db.get(inst.card_def_id)
                            .map(|d| d.name.as_str())
                            .unwrap_or("?")
                    )
                }
                crate::action::Action::PlayLand { object_id } => {
                    let inst = &state.objects[object_id];
                    format!(
                        "Play {}",
                        db.get(inst.card_def_id)
                            .map(|d| d.name.as_str())
                            .unwrap_or("?")
                    )
                }
                crate::action::Action::OrderTriggers { ordering } => {
                    format!("Order {} triggers", ordering.len())
                }
                other => format!("{}", other),
            };
            eprintln!(
                "T{} {:?} P{}: {} (life: {}/{})",
                state.turn_number,
                state.phase,
                player,
                action_name,
                state.players[0].life,
                state.players[1].life,
            );
        }

        rules::apply_action(state, &action);
        actions_taken += 1;

        if actions_taken.is_multiple_of(10) {
            rules::check_state_based_actions(state);
        }
    }

    GameResult {
        winner: state.winner,
        turns: state.turn_number,
        actions_taken,
        final_life: [state.players[0].life, state.players[1].life],
    }
}

/// Run many Commander games in parallel and aggregate results.
pub fn simulate_commander(
    card_db: &CardDatabase,
    deck0: &[CardId],
    deck1: &[CardId],
    commander0: CardId,
    commander1: CardId,
    strategy0: &(dyn Strategy + Send + Sync),
    strategy1: &(dyn Strategy + Send + Sync),
    num_games: u64,
) -> SimulationResults {
    let db = Arc::new(card_db.clone());
    let p0_wins = AtomicU64::new(0);
    let p1_wins = AtomicU64::new(0);
    let draws = AtomicU64::new(0);
    let total_turns = AtomicU64::new(0);
    let total_actions = AtomicU64::new(0);

    (0..num_games).into_par_iter().for_each(|_| {
        let result = run_commander_game_inner(
            Arc::clone(&db),
            deck0,
            deck1,
            commander0,
            commander1,
            strategy0,
            strategy1,
            false,
        );

        match result.winner {
            Some(0) => {
                p0_wins.fetch_add(1, Ordering::Relaxed);
            }
            Some(1) => {
                p1_wins.fetch_add(1, Ordering::Relaxed);
            }
            _ => {
                draws.fetch_add(1, Ordering::Relaxed);
            }
        }
        total_turns.fetch_add(result.turns as u64, Ordering::Relaxed);
        total_actions.fetch_add(result.actions_taken as u64, Ordering::Relaxed);
    });

    let total = num_games;
    SimulationResults {
        total_games: total,
        player0_wins: p0_wins.load(Ordering::Relaxed),
        player1_wins: p1_wins.load(Ordering::Relaxed),
        draws: draws.load(Ordering::Relaxed),
        avg_turns: total_turns.load(Ordering::Relaxed) as f64 / total as f64,
        avg_actions: total_actions.load(Ordering::Relaxed) as f64 / total as f64,
    }
}

/// Run many games in parallel and aggregate results.
pub fn simulate(
    card_db: &CardDatabase,
    deck0: &[CardId],
    deck1: &[CardId],
    strategy0: &(dyn Strategy + Send + Sync),
    strategy1: &(dyn Strategy + Send + Sync),
    num_games: u64,
) -> SimulationResults {
    let db = Arc::new(card_db.clone());
    let p0_wins = AtomicU64::new(0);
    let p1_wins = AtomicU64::new(0);
    let draws = AtomicU64::new(0);
    let total_turns = AtomicU64::new(0);
    let total_actions = AtomicU64::new(0);

    (0..num_games).into_par_iter().for_each(|_| {
        let result = run_game_inner(Arc::clone(&db), deck0, deck1, strategy0, strategy1, false);

        match result.winner {
            Some(0) => {
                p0_wins.fetch_add(1, Ordering::Relaxed);
            }
            Some(1) => {
                p1_wins.fetch_add(1, Ordering::Relaxed);
            }
            _ => {
                draws.fetch_add(1, Ordering::Relaxed);
            }
        }
        total_turns.fetch_add(result.turns as u64, Ordering::Relaxed);
        total_actions.fetch_add(result.actions_taken as u64, Ordering::Relaxed);
    });

    let total = num_games;
    SimulationResults {
        total_games: total,
        player0_wins: p0_wins.load(Ordering::Relaxed),
        player1_wins: p1_wins.load(Ordering::Relaxed),
        draws: draws.load(Ordering::Relaxed),
        avg_turns: total_turns.load(Ordering::Relaxed) as f64 / total as f64,
        avg_actions: total_actions.load(Ordering::Relaxed) as f64 / total as f64,
    }
}

// ---------------------------------------------------------------------------
// Goldfish mode — solitaire simulation against a passive opponent
// ---------------------------------------------------------------------------

/// Maximum turns before a goldfish game is declared a draw (terminal state).
/// Games that haven't ended by turn 20 are treated as terminal — this applies
/// to both Standard and Commander goldfish.
const GOLDFISH_MAX_TURNS: u32 = 20;

/// Maximum actions per goldfish game (lower bound since opponent does nothing).
const GOLDFISH_MAX_ACTIONS: u32 = 10_000;

/// Aggregate results from goldfish simulation.
///
/// Tracks kill-turn distribution in addition to standard win/loss stats.
/// Since the opponent takes no actions, the key metric is how quickly
/// the deck can win — the "goldfish kill turn".
#[derive(Debug, Clone)]
pub struct GoldfishResults {
    pub total_games: u64,
    /// Games where the pilot (player 0) reduced the goldfish to 0 life.
    pub wins: u64,
    /// Games where the pilot (player 0) lost — e.g. self-inflicted life loss,
    /// decking out, or an effect that causes the pilot to lose. Should be rare
    /// against a passive opponent, but tracked for completeness.
    pub losses: u64,
    /// Games that hit the turn/action limit without either player winning.
    pub draws: u64,
    pub avg_kill_turn: f64,
    pub fastest_kill: u32,
    pub slowest_kill: u32,
    pub avg_actions: f64,
    /// Kill-turn distribution: index = turn number, value = number of wins on that turn.
    /// Index 0 is unused (games start at turn 1).
    pub kill_turn_distribution: Vec<u64>,
}

impl GoldfishResults {
    pub fn win_rate(&self) -> f64 {
        self.wins as f64 / self.total_games as f64
    }

    pub fn display(&self) {
        println!("=== Goldfish Results ===");
        println!("Total games: {}", self.total_games);
        println!("Wins: {} ({:.1}%)", self.wins, self.win_rate() * 100.0);
        println!("Draws (timeout): {}", self.draws);
        if self.wins > 0 {
            println!("Avg kill turn: {:.2}", self.avg_kill_turn);
            println!("Fastest kill: T{}", self.fastest_kill);
            println!("Slowest kill: T{}", self.slowest_kill);
            println!("Avg actions/game: {:.1}", self.avg_actions);
            println!("Kill turn distribution:");
            for (turn, &count) in self.kill_turn_distribution.iter().enumerate() {
                if count > 0 {
                    let pct = count as f64 / self.wins as f64 * 100.0;
                    println!("  T{}: {} ({:.1}%)", turn, count, pct);
                }
            }
        }
    }
}

/// Run a single goldfish game: player 0 plays the deck under test,
/// player 1 uses GoldfishStrategy (does nothing).
///
/// Uses tighter limits than normal games since the goldfish opponent
/// adds no complexity.
pub fn run_goldfish_game(
    card_db: &CardDatabase,
    deck: &[CardId],
    strategy: &dyn Strategy,
) -> GameResult {
    let db = Arc::new(card_db.clone());
    // The goldfish opponent uses the same deck (it won't play any cards).
    run_goldfish_game_inner(db, deck, strategy, false)
}

/// Run a single goldfish game with verbose tracing.
pub fn run_goldfish_game_verbose(
    card_db: &CardDatabase,
    deck: &[CardId],
    strategy: &dyn Strategy,
) -> GameResult {
    let db = Arc::new(card_db.clone());
    run_goldfish_game_inner(db, deck, strategy, true)
}

fn run_goldfish_game_inner(
    card_db: Arc<CardDatabase>,
    deck: &[CardId],
    strategy: &dyn Strategy,
    verbose: bool,
) -> GameResult {
    let goldfish = GoldfishStrategy;
    let mut state = GameState::new(2);
    state.card_db = Some(card_db);

    // Both players get the same deck — the goldfish won't use its cards.
    rules::setup_game(&mut state, deck, deck);

    let mut actions_taken: u32 = 0;

    while !state.game_over
        && state.turn_number <= GOLDFISH_MAX_TURNS
        && actions_taken < GOLDFISH_MAX_ACTIONS
    {
        let player = state.priority_player;
        let actions = legal_actions(&state);

        if actions.is_empty()
            || (actions.len() == 1 && actions[0] == crate::action::Action::PassPriority)
        {
            rules::apply_action(&mut state, &crate::action::Action::PassPriority);
            actions_taken += 1;
            continue;
        }

        // Player 0 uses the provided strategy; player 1 is the goldfish.
        let active_strategy: &dyn Strategy = if player == 0 { strategy } else { &goldfish };
        let action = active_strategy.choose_action(&state, player);

        if verbose && actions_taken < 200 {
            let db = state.card_db();
            let action_name = match &action {
                crate::action::Action::CastSpell { object_id, .. } => {
                    let inst = &state.objects[object_id];
                    format!(
                        "Cast {}",
                        db.get(inst.card_def_id)
                            .map(|d| d.name.as_str())
                            .unwrap_or("?")
                    )
                }
                crate::action::Action::PlayLand { object_id } => {
                    let inst = &state.objects[object_id];
                    format!(
                        "Play {}",
                        db.get(inst.card_def_id)
                            .map(|d| d.name.as_str())
                            .unwrap_or("?")
                    )
                }
                crate::action::Action::OrderTriggers { ordering } => {
                    format!("Order {} triggers", ordering.len())
                }
                other => format!("{}", other),
            };
            eprintln!(
                "T{} {:?} P{}: {} (life: {}/{})",
                state.turn_number,
                state.phase,
                player,
                action_name,
                state.players[0].life,
                state.players[1].life,
            );
        }

        rules::apply_action(&mut state, &action);
        actions_taken += 1;

        if actions_taken.is_multiple_of(10) {
            rules::check_state_based_actions(&mut state);
        }
    }

    GameResult {
        winner: state.winner,
        turns: state.turn_number,
        actions_taken,
        final_life: [state.players[0].life, state.players[1].life],
    }
}

/// Maximum turns for commander goldfish games (terminal state).
/// Same as standard goldfish — games rarely reach this point.
const COMMANDER_GOLDFISH_MAX_TURNS: u32 = 20;

/// Run a single goldfish game in Commander format.
///
/// Player 0 uses the provided strategy; player 1 is a passive goldfish.
/// Commander-specific rules apply: 40 starting life, command zone,
/// commander tax, commander damage, and commander redirect.
pub fn run_commander_goldfish_game(
    card_db: &CardDatabase,
    deck: &[CardId],
    commander: CardId,
    strategy: &dyn Strategy,
) -> GameResult {
    let db = Arc::new(card_db.clone());
    run_commander_goldfish_game_inner(db, deck, commander, strategy, false)
}

fn run_commander_goldfish_game_inner(
    card_db: Arc<CardDatabase>,
    deck: &[CardId],
    commander: CardId,
    strategy: &dyn Strategy,
    verbose: bool,
) -> GameResult {
    let goldfish = GoldfishStrategy;
    let mut state = GameState::new_commander(2);
    state.card_db = Some(card_db);

    // Both players get the same deck/commander — the goldfish won't play cards.
    rules::setup_commander_game(&mut state, deck, deck, commander, commander);

    let mut actions_taken: u32 = 0;

    while !state.game_over
        && state.turn_number <= COMMANDER_GOLDFISH_MAX_TURNS
        && actions_taken < GOLDFISH_MAX_ACTIONS
    {
        let player = state.priority_player;
        let actions = legal_actions(&state);

        if actions.is_empty()
            || (actions.len() == 1 && actions[0] == crate::action::Action::PassPriority)
        {
            rules::apply_action(&mut state, &crate::action::Action::PassPriority);
            actions_taken += 1;
            continue;
        }

        // Player 0 uses the provided strategy; player 1 is the goldfish.
        let active_strategy: &dyn Strategy = if player == 0 { strategy } else { &goldfish };
        let action = active_strategy.choose_action(&state, player);

        if verbose && actions_taken < 200 {
            let db = state.card_db();
            let action_name = match &action {
                crate::action::Action::CastSpell { object_id, .. }
                | crate::action::Action::CastCommander { object_id, .. } => {
                    let inst = &state.objects[object_id];
                    format!(
                        "Cast {}",
                        db.get(inst.card_def_id)
                            .map(|d| d.name.as_str())
                            .unwrap_or("?")
                    )
                }
                crate::action::Action::PlayLand { object_id } => {
                    let inst = &state.objects[object_id];
                    format!(
                        "Play {}",
                        db.get(inst.card_def_id)
                            .map(|d| d.name.as_str())
                            .unwrap_or("?")
                    )
                }
                crate::action::Action::OrderTriggers { ordering } => {
                    format!("Order {} triggers", ordering.len())
                }
                other => format!("{}", other),
            };
            eprintln!(
                "T{} {:?} P{}: {} (life: {}/{})",
                state.turn_number,
                state.phase,
                player,
                action_name,
                state.players[0].life,
                state.players[1].life,
            );
        }

        rules::apply_action(&mut state, &action);
        actions_taken += 1;

        if actions_taken.is_multiple_of(10) {
            rules::check_state_based_actions(&mut state);
        }
    }

    GameResult {
        winner: state.winner,
        turns: state.turn_number,
        actions_taken,
        final_life: [state.players[0].life, state.players[1].life],
    }
}

/// Run many commander goldfish games in parallel and aggregate results.
pub fn simulate_commander_goldfish(
    card_db: &CardDatabase,
    deck: &[CardId],
    commander: CardId,
    strategy: &(dyn Strategy + Send + Sync),
    num_games: u64,
) -> GoldfishResults {
    let db = Arc::new(card_db.clone());
    let wins = AtomicU64::new(0);
    let losses = AtomicU64::new(0);
    let draws = AtomicU64::new(0);
    let total_kill_turns = AtomicU64::new(0);
    let total_actions = AtomicU64::new(0);
    let fastest = AtomicU64::new(u64::MAX);
    let slowest = AtomicU64::new(0);

    let distribution: Vec<AtomicU64> = (0..=COMMANDER_GOLDFISH_MAX_TURNS)
        .map(|_| AtomicU64::new(0))
        .collect();

    (0..num_games).into_par_iter().for_each(|_| {
        let result = run_commander_goldfish_game_inner(
            Arc::clone(&db), deck, commander, strategy, false,
        );

        total_actions.fetch_add(result.actions_taken as u64, Ordering::Relaxed);

        match result.winner {
            Some(0) => {
                wins.fetch_add(1, Ordering::Relaxed);
                let turn = result.turns;
                total_kill_turns.fetch_add(turn as u64, Ordering::Relaxed);
                if (turn as usize) < distribution.len() {
                    distribution[turn as usize].fetch_add(1, Ordering::Relaxed);
                }
                fastest.fetch_min(turn as u64, Ordering::Relaxed);
                slowest.fetch_max(turn as u64, Ordering::Relaxed);
            }
            Some(_) => {
                losses.fetch_add(1, Ordering::Relaxed);
            }
            None => {
                draws.fetch_add(1, Ordering::Relaxed);
            }
        }
    });

    let total_wins = wins.load(Ordering::Relaxed);
    let fast = fastest.load(Ordering::Relaxed);
    let slow = slowest.load(Ordering::Relaxed);

    let kill_turn_dist: Vec<u64> = distribution
        .iter()
        .map(|a| a.load(Ordering::Relaxed))
        .collect();

    GoldfishResults {
        total_games: num_games,
        wins: total_wins,
        losses: losses.load(Ordering::Relaxed),
        draws: draws.load(Ordering::Relaxed),
        avg_kill_turn: if total_wins > 0 {
            total_kill_turns.load(Ordering::Relaxed) as f64 / total_wins as f64
        } else {
            0.0
        },
        fastest_kill: if total_wins > 0 { fast as u32 } else { 0 },
        slowest_kill: if total_wins > 0 { slow as u32 } else { 0 },
        avg_actions: total_actions.load(Ordering::Relaxed) as f64 / num_games as f64,
        kill_turn_distribution: kill_turn_dist,
    }
}

/// Run many goldfish games in parallel and aggregate results with kill-turn distribution.
///
/// Goldfish simulation measures the fastest possible win speed for a deck by
/// playing against an opponent who takes no actions (no blocking, no spells).
/// This produces lower branching complexity and faster convergence than
/// a full two-player simulation.
pub fn simulate_goldfish(
    card_db: &CardDatabase,
    deck: &[CardId],
    strategy: &(dyn Strategy + Send + Sync),
    num_games: u64,
) -> GoldfishResults {
    let db = Arc::new(card_db.clone());
    let wins = AtomicU64::new(0);
    let losses = AtomicU64::new(0);
    let draws = AtomicU64::new(0);
    let total_kill_turns = AtomicU64::new(0);
    let total_actions = AtomicU64::new(0);
    let fastest = AtomicU64::new(u64::MAX);
    let slowest = AtomicU64::new(0);

    // Kill-turn distribution buckets (one per turn up to GOLDFISH_MAX_TURNS).
    let distribution: Vec<AtomicU64> = (0..=GOLDFISH_MAX_TURNS)
        .map(|_| AtomicU64::new(0))
        .collect();

    (0..num_games).into_par_iter().for_each(|_| {
        let result = run_goldfish_game_inner(Arc::clone(&db), deck, strategy, false);

        total_actions.fetch_add(result.actions_taken as u64, Ordering::Relaxed);

        match result.winner {
            Some(0) => {
                wins.fetch_add(1, Ordering::Relaxed);
                let turn = result.turns;
                total_kill_turns.fetch_add(turn as u64, Ordering::Relaxed);
                if (turn as usize) < distribution.len() {
                    distribution[turn as usize].fetch_add(1, Ordering::Relaxed);
                }
                // Update fastest/slowest atomically
                fastest.fetch_min(turn as u64, Ordering::Relaxed);
                slowest.fetch_max(turn as u64, Ordering::Relaxed);
            }
            Some(_) => {
                losses.fetch_add(1, Ordering::Relaxed);
            }
            None => {
                draws.fetch_add(1, Ordering::Relaxed);
            }
        }
    });

    let total_wins = wins.load(Ordering::Relaxed);
    let fast = fastest.load(Ordering::Relaxed);
    let slow = slowest.load(Ordering::Relaxed);

    let kill_turn_dist: Vec<u64> = distribution
        .iter()
        .map(|a| a.load(Ordering::Relaxed))
        .collect();

    GoldfishResults {
        total_games: num_games,
        wins: total_wins,
        losses: losses.load(Ordering::Relaxed),
        draws: draws.load(Ordering::Relaxed),
        avg_kill_turn: if total_wins > 0 {
            total_kill_turns.load(Ordering::Relaxed) as f64 / total_wins as f64
        } else {
            0.0
        },
        fastest_kill: if total_wins > 0 { fast as u32 } else { 0 },
        slowest_kill: if total_wins > 0 { slow as u32 } else { 0 },
        avg_actions: total_actions.load(Ordering::Relaxed) as f64 / num_games as f64,
        kill_turn_distribution: kill_turn_dist,
    }
}
