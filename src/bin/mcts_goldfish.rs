//! MCTS Goldfish — Solitaire optimization using Monte Carlo Tree Search
//!
//! Runs MCTS at each decision point to find near-optimal play for goldfish
//! (solitaire) games. Unlike MCCFR which trains a policy offline, MCTS
//! does online planning: it builds a search tree from the current position
//! each time it needs to make a decision.
//!
//! This is appropriate for single-agent optimization where the goal is to
//! minimize kill turn, rather than finding Nash equilibria for adversarial
//! play.
//!
//! Usage:
//!   cargo run --release --bin mcts_goldfish
//!
//! Options (via environment variables):
//!   ITERATIONS=500    MCTS iterations per decision (default: 500)
//!   EXPLORE=1.0       UCB1 exploration constant (default: 1.0)
//!   DEPTH=0           Max tree depth, 0=unlimited (default: 0)
//!   THREADS=1         Threads per MCTS decision, root parallelization (default: 1)
//!   GAMES=100         Number of games to simulate (default: 100)
//!   DECK=red          Deck: "red", "green", "kinnan", "brimaz", "ashcoat" (default: red)
//!   FORMAT=standard   Format: "standard" or "commander" (default: auto-detect)

use std::io::Write;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

use mtg_gto::card::sample;
use mtg_gto::game::CardDatabase;
use mtg_gto::simulation::{
    simulate_goldfish, simulate_commander_goldfish, GoldfishResults,
    simulate_mcts_goldfish_with_progress, simulate_mcts_commander_goldfish_with_progress,
    run_mcts_goldfish_game, run_mcts_commander_goldfish_game,
};
use mtg_gto::solver::mcts::MctsConfig;
use mtg_gto::strategy::GreedyStrategy;

fn main() {
    // Configure rayon thread pool with larger stack for deep MCTS rollouts
    rayon::ThreadPoolBuilder::new()
        .stack_size(8 * 1024 * 1024)
        .build_global()
        .expect("Failed to configure rayon thread pool");

    let iterations: u32 = std::env::var("ITERATIONS")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(500);
    let explore: f64 = std::env::var("EXPLORE")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(1.0);
    let max_depth: u32 = std::env::var("DEPTH")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(0);
    let num_games: u64 = std::env::var("GAMES")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(100);
    let num_threads: u32 = std::env::var("THREADS")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(1);
    let deck_name = std::env::var("DECK").unwrap_or_else(|_| "red".to_string());

    let db = sample::build_sample_db();

    let config = MctsConfig {
        iterations_per_move: iterations,
        exploration_constant: explore,
        max_tree_depth: max_depth,
        max_rollout_actions: 5_000,
        num_threads,
    };

    // Determine deck and format
    let is_commander = matches!(deck_name.as_str(), "kinnan" | "brimaz" | "ashcoat");

    println!("MCTS Goldfish Optimizer");
    println!("=======================");
    println!("Deck:       {}", deck_name);
    println!("Format:     {}", if is_commander { "Commander" } else { "Standard" });
    println!("MCTS iters: {} per decision", iterations);
    println!("Explore C:  {:.2}", explore);
    println!("Tree depth: {}", if max_depth == 0 { "unlimited".to_string() } else { format!("{}", max_depth) });
    println!("Threads:    {}{}", num_threads.max(1), if num_threads <= 1 { " (single-threaded)" } else { " (root parallelization)" });
    println!("Games:      {}", num_games);
    println!();

    if is_commander {
        run_commander_goldfish(&db, &deck_name, &config, num_games);
    } else {
        run_standard_goldfish(&db, &deck_name, &config, num_games);
    }
}

fn run_standard_goldfish(
    db: &CardDatabase,
    deck_name: &str,
    config: &MctsConfig,
    num_games: u64,
) {
    let deck = match deck_name {
        "green" => sample::green_stompy_deck(),
        _ => sample::red_aggro_deck(),
    };

    // ── 1. Greedy baseline ─────────────────────────────────────────────
    println!("Running Greedy baseline ({} games)...", num_games);
    let t0 = Instant::now();
    let greedy_results = simulate_goldfish(db, &deck, &GreedyStrategy, num_games);
    let greedy_time = t0.elapsed();
    println!("Greedy baseline complete in {:.1}s\n", greedy_time.as_secs_f64());

    // ── 2. MCTS simulation ─────────────────────────────────────────────
    println!("Running MCTS goldfish ({} games, {} iters/decision)...", num_games, config.iterations_per_move);
    let t0 = Instant::now();
    let progress = Arc::new(AtomicU64::new(0));
    let done = Arc::new(AtomicBool::new(false));
    let printer = spawn_progress_thread(progress.clone(), done.clone(), num_games, t0);
    let mcts_results = simulate_mcts_goldfish_with_progress(db, &deck, config, num_games, &progress);
    done.store(true, Ordering::Relaxed);
    let _ = printer.join();
    let mcts_time = t0.elapsed();
    // Clear the progress line, then print the final summary
    print!("\r{: <60}\r", "");
    let _ = std::io::stdout().flush();
    println!("MCTS simulation complete in {:.1}s\n", mcts_time.as_secs_f64());

    // ── 3. Comparison ──────────────────────────────────────────────────
    println!("Strategy Comparison");
    println!("───────────────────");
    print_strategy_row("Greedy", &greedy_results);
    print_mcts_strategy_row("MCTS", &mcts_results);
    println!();

    let delta = greedy_results.avg_kill_turn - mcts_results.avg_kill_turn;
    println!(
        "MCTS improves by {:.2} turns on average (Greedy T{:.2} → MCTS T{:.2})",
        delta, greedy_results.avg_kill_turn, mcts_results.avg_kill_turn,
    );
    println!();

    // ── 4. Kill turn distribution comparison ───────────────────────────
    println!("Kill Turn Distribution (cumulative P(win by turn X))");
    println!("────────────────────────────────────────────────────");
    print_distribution_comparison(&greedy_results, &mcts_results);
    println!();

    // ── 5. Fastest win sequence ──────────────────────────────────────
    print_fastest_sequence(&mcts_results, config);

    // ── 6. Sample game trace ───────────────────────────────────────────
    println!("Sample Game Trace (MCTS)");
    println!("────────────────────────");
    println!("(Actions logged to stderr)\n");

    let result = run_mcts_goldfish_game(db, &deck, config, true);
    println!(
        "Result: {} on T{} ({} actions), life: {}/{}",
        if result.won { "WIN" } else { "DRAW" },
        result.kill_turn,
        result.actions_taken,
        result.final_life[0],
        result.final_life[1],
    );
    if !result.decision_stats.is_empty() {
        println!("\nKey decisions ({} total):", result.decision_stats.len());
        for (i, stat) in result.decision_stats.iter().enumerate().take(30) {
            println!(
                "  #{:<3} T{} {:?}: {} (of {} options, {}/{} visits, Q={:.3})",
                i + 1,
                stat.turn,
                stat.phase,
                stat.action_description,
                stat.num_legal_actions,
                stat.best_action_visits,
                config.iterations_per_move,
                stat.best_action_avg_reward,
            );
        }
    }
}

fn run_commander_goldfish(
    db: &CardDatabase,
    deck_name: &str,
    config: &MctsConfig,
    num_games: u64,
) {
    let (deck, commander, _tutor_targets) = match deck_name {
        "brimaz" => {
            let (d, c) = sample::brimaz_commander_deck();
            (d, c, Vec::new())
        }
        "ashcoat" => {
            let (d, c) = sample::ashcoat_commander_deck();
            (d, c, Vec::new())
        }
        _ => sample::kinnan_commander_deck(),
    };

    let commander_name = db.get(commander).map(|d| d.name.as_str()).unwrap_or("?");
    println!("Commander: {}\n", commander_name);

    // ── 1. Greedy baseline ─────────────────────────────────────────────
    println!("Running Greedy baseline ({} games)...", num_games);
    let t0 = Instant::now();
    let greedy_results = simulate_commander_goldfish(db, &deck, commander, &GreedyStrategy, num_games);
    let greedy_time = t0.elapsed();
    println!("Greedy baseline complete in {:.1}s\n", greedy_time.as_secs_f64());

    // ── 2. MCTS simulation ─────────────────────────────────────────────
    println!("Running MCTS goldfish ({} games, {} iters/decision)...", num_games, config.iterations_per_move);
    let t0 = Instant::now();
    let progress = Arc::new(AtomicU64::new(0));
    let done = Arc::new(AtomicBool::new(false));
    let printer = spawn_progress_thread(progress.clone(), done.clone(), num_games, t0);
    let mcts_results = simulate_mcts_commander_goldfish_with_progress(db, &deck, commander, config, num_games, &progress);
    done.store(true, Ordering::Relaxed);
    let _ = printer.join();
    let mcts_time = t0.elapsed();
    // Clear the progress line, then print the final summary
    print!("\r{: <60}\r", "");
    let _ = std::io::stdout().flush();
    println!("MCTS simulation complete in {:.1}s\n", mcts_time.as_secs_f64());

    // ── 3. Comparison ──────────────────────────────────────────────────
    println!("Strategy Comparison");
    println!("───────────────────");
    print_strategy_row("Greedy", &greedy_results);
    print_mcts_strategy_row("MCTS", &mcts_results);
    println!();

    let delta = greedy_results.avg_kill_turn - mcts_results.avg_kill_turn;
    println!(
        "MCTS improves by {:.2} turns on average (Greedy T{:.2} → MCTS T{:.2})",
        delta, greedy_results.avg_kill_turn, mcts_results.avg_kill_turn,
    );
    println!();

    // ── 4. Kill turn distribution comparison ───────────────────────────
    println!("Kill Turn Distribution (cumulative P(win by turn X))");
    println!("────────────────────────────────────────────────────");
    print_distribution_comparison(&greedy_results, &mcts_results);
    println!();

    // ── 5. Fastest win sequence ──────────────────────────────────────
    print_fastest_sequence(&mcts_results, config);

    // ── 6. Sample game trace ───────────────────────────────────────────
    println!("Sample Game Trace (MCTS)");
    println!("────────────────────────");
    println!("(Actions logged to stderr)\n");

    let result = run_mcts_commander_goldfish_game(db, &deck, commander, config, true);
    println!(
        "Result: {} on T{} ({} actions), life: {}/{}",
        if result.won { "WIN" } else { "DRAW" },
        result.kill_turn,
        result.actions_taken,
        result.final_life[0],
        result.final_life[1],
    );
    if !result.decision_stats.is_empty() {
        println!("\nKey decisions ({} total):", result.decision_stats.len());
        for (i, stat) in result.decision_stats.iter().enumerate().take(30) {
            println!(
                "  #{:<3} T{} {:?}: {} (of {} options, {}/{} visits, Q={:.3})",
                i + 1,
                stat.turn,
                stat.phase,
                stat.action_description,
                stat.num_legal_actions,
                stat.best_action_visits,
                config.iterations_per_move,
                stat.best_action_avg_reward,
            );
        }
    }
}

fn print_fastest_sequence(
    r: &mtg_gto::solver::mcts::MctsGoldfishResults,
    config: &MctsConfig,
) {
    if r.fastest_sequence.is_empty() {
        return;
    }
    println!("Fastest Win Sequence (T{} kill)", r.fastest_kill);
    println!("─────────────────────────────────");
    for (i, stat) in r.fastest_sequence.iter().enumerate() {
        println!(
            "  #{:<3} T{} {:?}: {} (of {} options, {}/{} visits, Q={:.3})",
            i + 1,
            stat.turn,
            stat.phase,
            stat.action_description,
            stat.num_legal_actions,
            stat.best_action_visits,
            config.iterations_per_move,
            stat.best_action_avg_reward,
        );
    }
    println!();
}

fn print_strategy_row(name: &str, r: &GoldfishResults) {
    println!(
        "  {:<8} win={:>5.1}%  avg_kill=T{:<5.2}  fastest=T{:<3}  slowest=T{:<3}  draws={}",
        name,
        r.win_rate() * 100.0,
        r.avg_kill_turn,
        r.fastest_kill,
        r.slowest_kill,
        r.draws,
    );
}

fn print_mcts_strategy_row(
    name: &str,
    r: &mtg_gto::solver::mcts::MctsGoldfishResults,
) {
    println!(
        "  {:<8} win={:>5.1}%  avg_kill=T{:<5.2}  fastest=T{:<3}  slowest=T{:<3}  draws={}",
        name,
        r.win_rate() * 100.0,
        r.avg_kill_turn,
        r.fastest_kill,
        r.slowest_kill,
        r.draws,
    );
}

fn print_distribution_comparison(
    greedy: &GoldfishResults,
    mcts: &mtg_gto::solver::mcts::MctsGoldfishResults,
) {
    println!("  Turn │   Greedy   │    MCTS");
    println!("  ─────┼────────────┼────────────");

    let max_turn = [greedy.slowest_kill, mcts.slowest_kill, 20]
        .into_iter()
        .max()
        .unwrap_or(20) as usize;

    let mut cum_g = 0u64;
    let mut cum_m = 0u64;

    for turn in 1..=max_turn {
        let gc = greedy.kill_turn_distribution.get(turn).copied().unwrap_or(0);
        let mc = mcts.kill_turn_distribution.get(turn).copied().unwrap_or(0);
        cum_g += gc;
        cum_m += mc;

        if cum_g == 0 && cum_m == 0 {
            continue;
        }

        let pg = cum_g as f64 / greedy.total_games as f64 * 100.0;
        let pm = cum_m as f64 / mcts.total_games as f64 * 100.0;

        println!(
            "  T{:<3} │  {:>5.1}%     │  {:>5.1}%",
            turn, pg, pm,
        );
    }
}

/// Spawn a background thread that prints MCTS game-level progress every 2 seconds.
///
/// Returns the thread handle so the caller can join after the simulation completes.
fn spawn_progress_thread(
    progress: Arc<AtomicU64>,
    done: Arc<AtomicBool>,
    total: u64,
    start: Instant,
) -> std::thread::JoinHandle<()> {
    std::thread::spawn(move || {
        loop {
            std::thread::sleep(Duration::from_secs(2));
            if done.load(Ordering::Relaxed) {
                break;
            }
            let completed = progress.load(Ordering::Relaxed);
            let elapsed = start.elapsed().as_secs_f64();
            if completed > 0 {
                let eta = elapsed / completed as f64 * (total - completed) as f64;
                print!(
                    "\r  game {}/{} | {:.1}s elapsed | ETA {:.0}s",
                    completed, total, elapsed, eta,
                );
            } else {
                print!("\r  game 0/{} | {:.1}s elapsed | ETA ...", total, elapsed);
            }
            let _ = std::io::stdout().flush();
        }
    })
}
