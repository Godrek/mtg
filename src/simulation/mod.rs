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
    run_game_loop(&mut state, strategy0, strategy1, verbose)
}

/// Run a single game with a fixed random seed for deterministic replay.
///
/// Given the same seed, decks, and deterministic strategies, the game will
/// produce identical results every time. Useful for debugging and regression
/// testing.
pub fn run_game_seeded(
    card_db: &CardDatabase,
    deck0: &[CardId],
    deck1: &[CardId],
    strategy0: &dyn Strategy,
    strategy1: &dyn Strategy,
    seed: u64,
) -> GameResult {
    let db = Arc::new(card_db.clone());
    let mut state = GameState::new(2);
    state.card_db = Some(db);
    rules::setup_game_seeded(&mut state, deck0, deck1, seed);
    run_game_loop(&mut state, strategy0, strategy1, false)
}

/// Run a single Commander goldfish game with a fixed random seed.
pub fn run_commander_goldfish_game_seeded(
    card_db: &CardDatabase,
    deck: &[CardId],
    commander: CardId,
    strategy: &dyn Strategy,
    seed: u64,
) -> GameResult {
    let db = Arc::new(card_db.clone());
    let mut state = GameState::new_commander(2);
    state.card_db = Some(db);
    rules::setup_commander_game_seeded(&mut state, deck, deck, commander, commander, seed);
    run_goldfish_loop(&mut state, strategy, false)
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
            log_action(state, &action, player);
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
pub fn run_goldfish_game(
    card_db: &CardDatabase,
    deck: &[CardId],
    strategy: &dyn Strategy,
) -> GameResult {
    let db = Arc::new(card_db.clone());
    init_and_run_goldfish(db, strategy, |state| {
        rules::setup_game(state, deck, deck);
    })
}

/// Run a single goldfish game with verbose tracing.
pub fn run_goldfish_game_verbose(
    card_db: &CardDatabase,
    deck: &[CardId],
    strategy: &dyn Strategy,
) -> GameResult {
    let db = Arc::new(card_db.clone());
    let mut state = GameState::new(2);
    state.card_db = Some(db);
    rules::setup_game(&mut state, deck, deck);
    run_goldfish_loop(&mut state, strategy, true)
}

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
    init_and_run_goldfish(db, strategy, |state| {
        *state = GameState::new_commander(2);
        // card_db is set by the caller before this closure; re-set after replacing state
        rules::setup_commander_game(state, deck, deck, commander, commander);
    })
}

/// Run a single Commander goldfish game with verbose tracing.
pub fn run_commander_goldfish_game_verbose(
    card_db: &CardDatabase,
    deck: &[CardId],
    commander: CardId,
    strategy: &dyn Strategy,
) -> GameResult {
    let db = Arc::new(card_db.clone());
    let mut state = GameState::new_commander(2);
    state.card_db = Some(db);
    rules::setup_commander_game(&mut state, deck, deck, commander, commander);
    run_goldfish_loop(&mut state, strategy, true)
}

/// Initialize a game state and run the goldfish loop.
///
/// The `setup` closure receives a `&mut GameState` with `card_db` already set.
/// For Commander, it should replace the state with `GameState::new_commander`
/// and call the appropriate setup function.
fn init_and_run_goldfish(
    card_db: Arc<CardDatabase>,
    strategy: &dyn Strategy,
    setup: impl FnOnce(&mut GameState),
) -> GameResult {
    let mut state = GameState::new(2);
    state.card_db = Some(card_db.clone());
    setup(&mut state);
    // Ensure card_db is set after setup (Commander setup replaces the state)
    if state.card_db.is_none() {
        state.card_db = Some(card_db);
    }
    run_goldfish_loop(&mut state, strategy, false)
}

/// Shared goldfish game loop for both Standard and Commander formats.
///
/// Player 0 uses the provided strategy; player 1 is a passive goldfish
/// that always passes priority.
fn run_goldfish_loop(
    state: &mut GameState,
    strategy: &dyn Strategy,
    verbose: bool,
) -> GameResult {
    let mut actions_taken: u32 = 0;

    while !state.game_over
        && state.turn_number <= GOLDFISH_MAX_TURNS
        && actions_taken < GOLDFISH_MAX_ACTIONS
    {
        // Fast-forward the goldfish's entire turn without calling legal_actions
        if state.active_player != 0 {
            actions_taken += rules::fast_forward_goldfish_turn(state);
            continue;
        }

        let player = state.priority_player;
        let actions = legal_actions(state);

        if actions.is_empty()
            || (actions.len() == 1 && actions[0] == crate::action::Action::PassPriority)
        {
            rules::apply_action(state, &crate::action::Action::PassPriority);
            actions_taken += 1;
            continue;
        }

        let action = strategy.choose_action(state, player);

        if verbose && actions_taken < 200 {
            log_action(state, &action, player);
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
    aggregate_goldfish_results(num_games, |_| {
        let mut state = GameState::new(2);
        state.card_db = Some(Arc::clone(&db));
        rules::setup_game(&mut state, deck, deck);
        run_goldfish_loop(&mut state, strategy, false)
    })
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
    aggregate_goldfish_results(num_games, |_| {
        let mut state = GameState::new_commander(2);
        state.card_db = Some(Arc::clone(&db));
        rules::setup_commander_game(&mut state, deck, deck, commander, commander);
        run_goldfish_loop(&mut state, strategy, false)
    })
}

/// Shared aggregation logic for goldfish simulations.
fn aggregate_goldfish_results(
    num_games: u64,
    run_one: impl Fn(u64) -> GameResult + Send + Sync,
) -> GoldfishResults {
    let wins = AtomicU64::new(0);
    let losses = AtomicU64::new(0);
    let draws = AtomicU64::new(0);
    let total_kill_turns = AtomicU64::new(0);
    let total_actions = AtomicU64::new(0);
    let fastest = AtomicU64::new(u64::MAX);
    let slowest = AtomicU64::new(0);

    let distribution: Vec<AtomicU64> = (0..=GOLDFISH_MAX_TURNS)
        .map(|_| AtomicU64::new(0))
        .collect();

    (0..num_games).into_par_iter().for_each(|i| {
        let result = run_one(i);

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

// ---------------------------------------------------------------------------
// MCTS goldfish — solitaire optimization using Monte Carlo Tree Search
// ---------------------------------------------------------------------------

use crate::solver::mcts::{self, MctsConfig, MctsGoldfishResults};

/// Run a single goldfish game using MCTS for player 0's decisions.
///
/// Unlike `run_goldfish_game()` which uses a fixed strategy, this runs
/// MCTS search at each decision point to find near-optimal play for
/// the given shuffle.
pub fn run_mcts_goldfish_game(
    card_db: &CardDatabase,
    deck: &[CardId],
    config: &MctsConfig,
    verbose: bool,
) -> mcts::MctsGameResult {
    let db = Arc::new(card_db.clone());
    let mut state = GameState::new(2);
    state.card_db = Some(db);
    rules::setup_game(&mut state, deck, deck);
    mcts::run_mcts_goldfish_game(&mut state, config, verbose, None)
}

/// Run a single Commander goldfish game using MCTS.
pub fn run_mcts_commander_goldfish_game(
    card_db: &CardDatabase,
    deck: &[CardId],
    commander: CardId,
    config: &MctsConfig,
    verbose: bool,
    tutor_targets: &[CardId],
) -> mcts::MctsGameResult {
    let db = Arc::new(card_db.clone());
    let mut state = GameState::new_commander(2);
    state.card_db = Some(db);
    rules::setup_commander_game(&mut state, deck, deck, commander, commander);
    rules::set_tutor_targets(&mut state, 0, tutor_targets);
    // Discover combos using only the commander + tutor targets (known combo
    // pieces). Using the full 99-card deck finds hundreds of spurious combos
    // and makes legal_actions() extremely slow.
    let mut combo_cards = vec![commander];
    combo_cards.extend_from_slice(tutor_targets);
    let (mut registry, _) = crate::combo_discovery::discover_and_register(
        card_db,
        &combo_cards,
        &crate::combo_discovery::DiscoveryConfig::default(),
    );
    crate::combo::register_ballista_win_combo(&mut registry);
    state.combo_registry = Some(Arc::new(registry));
    mcts::run_mcts_goldfish_game(&mut state, config, verbose, None)
}

/// Run many MCTS goldfish games in parallel and aggregate results.
///
/// Each game gets a fresh shuffle and runs MCTS at every decision point
/// for player 0. This measures how well MCTS-optimized play performs
/// across many random draws.
pub fn simulate_mcts_goldfish(
    card_db: &CardDatabase,
    deck: &[CardId],
    config: &MctsConfig,
    num_games: u64,
) -> MctsGoldfishResults {
    let db = Arc::new(card_db.clone());
    aggregate_mcts_goldfish_results(num_games, config, |_| {
        let mut state = GameState::new(2);
        state.card_db = Some(Arc::clone(&db));
        rules::setup_game(&mut state, deck, deck);
        state
    }, None, None)
}

/// Like [`simulate_mcts_goldfish`], but increments `progress` after each game
/// completes so a background thread can report progress.
pub fn simulate_mcts_goldfish_with_progress(
    card_db: &CardDatabase,
    deck: &[CardId],
    config: &MctsConfig,
    num_games: u64,
    progress: &AtomicU64,
    decisions: &AtomicU64,
) -> MctsGoldfishResults {
    let db = Arc::new(card_db.clone());
    aggregate_mcts_goldfish_results(num_games, config, |_| {
        let mut state = GameState::new(2);
        state.card_db = Some(Arc::clone(&db));
        rules::setup_game(&mut state, deck, deck);
        state
    }, Some(progress), Some(decisions))
}

/// Run many Commander MCTS goldfish games in parallel and aggregate results.
pub fn simulate_mcts_commander_goldfish(
    card_db: &CardDatabase,
    deck: &[CardId],
    commander: CardId,
    config: &MctsConfig,
    num_games: u64,
    tutor_targets: &[CardId],
) -> MctsGoldfishResults {
    let db = Arc::new(card_db.clone());
    let targets = tutor_targets.to_vec();
    // Discover combos using commander + tutor targets only
    let mut combo_cards = vec![commander];
    combo_cards.extend_from_slice(tutor_targets);
    let (mut registry, _) = crate::combo_discovery::discover_and_register(
        card_db,
        &combo_cards,
        &crate::combo_discovery::DiscoveryConfig::default(),
    );
    crate::combo::register_ballista_win_combo(&mut registry);
    let combo_reg = Arc::new(registry);
    aggregate_mcts_goldfish_results(num_games, config, |_| {
        let mut state = GameState::new_commander(2);
        state.card_db = Some(Arc::clone(&db));
        rules::setup_commander_game(&mut state, deck, deck, commander, commander);
        rules::set_tutor_targets(&mut state, 0, &targets);
        state.combo_registry = Some(Arc::clone(&combo_reg));
        state
    }, None, None)
}

/// Like [`simulate_mcts_commander_goldfish`], but increments `progress` after
/// each game completes so a background thread can report progress.
pub fn simulate_mcts_commander_goldfish_with_progress(
    card_db: &CardDatabase,
    deck: &[CardId],
    commander: CardId,
    config: &MctsConfig,
    num_games: u64,
    progress: &AtomicU64,
    decisions: &AtomicU64,
    tutor_targets: &[CardId],
) -> MctsGoldfishResults {
    let db = Arc::new(card_db.clone());
    let targets = tutor_targets.to_vec();
    // Discover combos using commander + tutor targets only
    let mut combo_cards = vec![commander];
    combo_cards.extend_from_slice(tutor_targets);
    let (mut registry, _) = crate::combo_discovery::discover_and_register(
        card_db,
        &combo_cards,
        &crate::combo_discovery::DiscoveryConfig::default(),
    );
    crate::combo::register_ballista_win_combo(&mut registry);
    let combo_reg = Arc::new(registry);
    aggregate_mcts_goldfish_results(num_games, config, |_| {
        let mut state = GameState::new_commander(2);
        state.card_db = Some(Arc::clone(&db));
        rules::setup_commander_game(&mut state, deck, deck, commander, commander);
        rules::set_tutor_targets(&mut state, 0, &targets);
        state.combo_registry = Some(Arc::clone(&combo_reg));
        state
    }, Some(progress), Some(decisions))
}

/// Shared aggregation logic for MCTS goldfish simulations.
///
/// If `progress` is provided, it is incremented (atomically) after each game
/// completes, allowing a background thread to report progress.
fn aggregate_mcts_goldfish_results(
    num_games: u64,
    config: &MctsConfig,
    make_state: impl Fn(u64) -> GameState + Send + Sync,
    progress: Option<&AtomicU64>,
    decisions: Option<&AtomicU64>,
) -> MctsGoldfishResults {
    use std::sync::Mutex;

    let wins = AtomicU64::new(0);
    let losses = AtomicU64::new(0);
    let draws = AtomicU64::new(0);
    let total_kill_turns = AtomicU64::new(0);
    let total_actions = AtomicU64::new(0);
    let fastest = AtomicU64::new(u64::MAX);
    let slowest = AtomicU64::new(0);
    let total_decisions = AtomicU64::new(0);
    // Use Mutex<f64> for exact floating-point accumulation (no ×1000 truncation).
    let total_reward = Mutex::new(0.0f64);
    // Track the decision sequence from the fastest winning game.
    let fastest_sequence: Mutex<Vec<mcts::DecisionStat>> = Mutex::new(Vec::new());

    let max_turn = 20u32;
    let distribution: Vec<AtomicU64> = (0..=max_turn)
        .map(|_| AtomicU64::new(0))
        .collect();

    (0..num_games).into_par_iter().for_each(|i| {
        let mut state = make_state(i);
        let result = mcts::run_mcts_goldfish_game(&mut state, config, false, decisions);

        total_actions.fetch_add(result.actions_taken as u64, Ordering::Relaxed);

        let n_decisions = result.decision_stats.len() as u64;
        total_decisions.fetch_add(n_decisions, Ordering::Relaxed);
        let avg_reward: f64 = if result.decision_stats.is_empty() {
            0.0
        } else {
            result.decision_stats.iter().map(|d| d.best_action_avg_reward).sum::<f64>()
                / result.decision_stats.len() as f64
        };
        *total_reward.lock().unwrap() += avg_reward;

        if result.won {
            wins.fetch_add(1, Ordering::Relaxed);
            let turn = result.kill_turn;
            total_kill_turns.fetch_add(turn as u64, Ordering::Relaxed);
            if (turn as usize) < distribution.len() {
                distribution[turn as usize].fetch_add(1, Ordering::Relaxed);
            }
            let prev_fastest = fastest.fetch_min(turn as u64, Ordering::Relaxed);
            slowest.fetch_max(turn as u64, Ordering::Relaxed);
            // If this game is the new fastest (or tied), save its decision sequence
            if (turn as u64) <= prev_fastest {
                *fastest_sequence.lock().unwrap() = result.decision_stats;
            }
        } else if result.final_life[0] <= 0 {
            losses.fetch_add(1, Ordering::Relaxed);
        } else {
            draws.fetch_add(1, Ordering::Relaxed);
        }

        if let Some(p) = progress {
            p.fetch_add(1, Ordering::Relaxed);
        }
    });

    let total_wins = wins.load(Ordering::Relaxed);
    let fast = fastest.load(Ordering::Relaxed);
    let slow = slowest.load(Ordering::Relaxed);
    let tot_decisions = total_decisions.load(Ordering::Relaxed);
    let tot_reward = *total_reward.lock().unwrap();

    let kill_turn_dist: Vec<u64> = distribution
        .iter()
        .map(|a| a.load(Ordering::Relaxed))
        .collect();

    MctsGoldfishResults {
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
        avg_decisions_per_game: if num_games > 0 {
            tot_decisions as f64 / num_games as f64
        } else {
            0.0
        },
        avg_best_reward: if num_games > 0 {
            tot_reward / num_games as f64
        } else {
            0.0
        },
        kill_turn_distribution: kill_turn_dist,
        fastest_sequence: fastest_sequence.into_inner().unwrap(),
    }
}

/// Format a game action as a human-readable string, resolving card names
/// from the game state's card database.
///
/// Shared by `log_action` (simulation verbose tracing) and MCTS decision
/// logging so that card-name resolution isn't duplicated.
pub fn format_action_name(state: &GameState, action: &crate::action::Action) -> String {
    let db = state.card_db();
    match action {
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
        crate::action::Action::DeclareAttackers { attackers } => {
            let names: Vec<String> = attackers
                .iter()
                .filter_map(|id| {
                    state.objects.get(id).and_then(|inst| {
                        db.get(inst.card_def_id).map(|d| d.name.clone())
                    })
                })
                .collect();
            if names.is_empty() {
                "Attack: none".to_string()
            } else {
                format!("Attack: {}", names.join(", "))
            }
        }
        crate::action::Action::ActivateAbility { object_id, ability_index, .. } => {
            let inst = &state.objects[object_id];
            format!(
                "Activate {} ability #{}",
                db.get(inst.card_def_id)
                    .map(|d| d.name.as_str())
                    .unwrap_or("?"),
                ability_index,
            )
        }
        crate::action::Action::ActivateMacro { combo_id } => {
            format!("Activate combo #{}", combo_id)
        }
        crate::action::Action::OrderTriggers { ordering } => {
            format!("Order {} triggers", ordering.len())
        }
        crate::action::Action::ChooseTutorTarget { card_id } => {
            format!(
                "Tutor for {}",
                db.get(*card_id)
                    .map(|d| d.name.as_str())
                    .unwrap_or("?")
            )
        }
        crate::action::Action::PassPriority => "Pass".to_string(),
        other => format!("{}", other),
    }
}

/// Return the card names in a player's hand.
pub fn format_hand(state: &GameState, player: PlayerIndex) -> Vec<String> {
    let db = state.card_db();
    state.players[player].hand.iter().filter_map(|oid| {
        state.objects.get(oid).and_then(|inst| {
            db.get(inst.card_def_id).map(|d| d.name.clone())
        })
    }).collect()
}

/// Log a game action to stderr for verbose tracing.
fn log_action(state: &GameState, action: &crate::action::Action, player: PlayerIndex) {
    let action_name = format_action_name(state, action);
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
