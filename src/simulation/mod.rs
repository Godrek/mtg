use rayon::prelude::*;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

use crate::action::legal_actions;
use crate::card::CardId;
use crate::game::{CardDatabase, GameState, PlayerIndex};
use crate::rules;
use crate::strategy::Strategy;

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

    let mut actions_taken: u32 = 0;

    while !state.game_over && state.turn_number <= MAX_TURNS && actions_taken < MAX_ACTIONS {
        let player = state.priority_player;
        let actions = legal_actions(&state);

        if actions.is_empty() || (actions.len() == 1 && actions[0] == crate::action::Action::PassPriority) {
            rules::apply_action(&mut state, &crate::action::Action::PassPriority);
            actions_taken += 1;
            continue;
        }

        let strategy: &dyn Strategy = if player == 0 { strategy0 } else { strategy1 };
        let action = strategy.choose_action(&state, player);

        if verbose && actions_taken < 200 {
            let db = state.card_db();
            let action_name = match &action {
                crate::action::Action::CastSpell { object_id, .. } => {
                    let inst = &state.objects[object_id];
                    format!("Cast {}", db.get(inst.card_def_id).map(|d| d.name.as_str()).unwrap_or("?"))
                }
                crate::action::Action::PlayLand { object_id } => {
                    let inst = &state.objects[object_id];
                    format!("Play {}", db.get(inst.card_def_id).map(|d| d.name.as_str()).unwrap_or("?"))
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

        // Periodic SBA check
        if actions_taken % 10 == 0 {
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
