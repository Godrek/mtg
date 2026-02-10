//! Commander Goldfish MCCFR Training
//!
//! Trains an MCCFR strategy for a commander deck in goldfish (solitaire) mode,
//! then reports:
//!   - Per-turn kill probability distribution
//!   - Cumulative win-by-turn-X probabilities
//!   - Comparison against Greedy and Random baselines
//!   - A sample game trace showing the trained strategy's decisions
//!
//! Usage:
//!   cargo run --release --bin commander_goldfish
//!
//! Options (via environment variables):
//!   ITERATIONS=100    Number of MCCFR training iterations (default: 100)
//!   DEPTH=10          Max tree depth per traversal (default: 10)
//!   GAMES=1000        Number of simulation games (default: 1000)
//!   DECK=kinnan       Deck to use: "kinnan" or "brimaz" (default: kinnan)

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;

use mtg_gto::card::sample;
use mtg_gto::game::{CardDatabase, GameState};
use mtg_gto::info_set::BucketedAbstraction;
use mtg_gto::rules;
use mtg_gto::simulation::{
    run_commander_goldfish_game_verbose, simulate_commander_goldfish, GoldfishResults,
};
use mtg_gto::solver::mccfr::{self, collect_policy_snapshots, McfrConfig};
use mtg_gto::strategy::{AbstractedMcfrStrategy, GreedyStrategy, RandomStrategy};

fn main() {
    let iterations: u32 = std::env::var("ITERATIONS")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(100);
    let max_depth: u32 = std::env::var("DEPTH")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(10);
    let num_games: u64 = std::env::var("GAMES")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(1000);
    let deck_name = std::env::var("DECK").unwrap_or_else(|_| "kinnan".to_string());

    let db = sample::build_sample_db();
    let (deck, commander) = match deck_name.as_str() {
        "brimaz" => sample::brimaz_commander_deck(),
        _ => sample::kinnan_commander_deck(),
    };

    let commander_name = db.get(commander).map(|d| d.name.as_str()).unwrap_or("?");

    println!("Commander Goldfish MCCFR Training");
    println!("==================================");
    println!("Deck:       {} ({})", deck_name, commander_name);
    println!("Iterations: {}", iterations);
    println!("Depth:      {}", max_depth);
    println!("Sim games:  {}", num_games);
    println!();

    // Build card name lookup for readable output
    let card_names = build_card_names(&db, &deck, commander);

    // ── 1. Train MCCFR ──────────────────────────────────────────────────
    println!("Training...");
    let mut state = GameState::new_commander(2);
    state.card_db = Some(Arc::new(db.clone()));
    rules::setup_commander_game(&mut state, &deck, &deck, commander, commander);

    let config = McfrConfig {
        max_depth,
        max_actions: 2000,
    };
    let abstraction = BucketedAbstraction;

    let t0 = Instant::now();
    let last_print = std::cell::Cell::new(Instant::now());
    let tables = mccfr::train_goldfish_with_progress(
        &state,
        iterations,
        &config,
        &abstraction,
        0,
        |iter, total, tables| {
            let now = Instant::now();
            // Print on first iteration, every 2 seconds, and on the last iteration.
            let should_print = iter == 1
                || iter == total
                || now.duration_since(last_print.get()).as_secs_f64() >= 2.0;
            if !should_print {
                return;
            }
            last_print.set(now);
            let elapsed = now.duration_since(t0).as_secs_f64();
            let info_sets = tables[0].num_info_sets();
            let exploit = mccfr::approximate_exploitability(tables);
            let eta = if iter > 0 {
                elapsed / iter as f64 * (total - iter) as f64
            } else {
                0.0
            };
            eprint!(
                "\r  iter {}/{} | {:.1}s elapsed | ETA {:.0}s | info_sets: {} | exploit: {:.6}   ",
                iter, total, elapsed, eta, info_sets, exploit,
            );
        },
    );
    let train_time = t0.elapsed();
    eprintln!();

    let stats = mccfr::training_stats(&tables);
    let exploit = mccfr::approximate_exploitability(&tables);

    println!("Training complete in {:.1}s", train_time.as_secs_f64());
    println!("  Info sets:      {}", stats.total_info_sets[0]);
    println!("  Total visits:   {}", stats.total_visits[0]);
    println!("  Exploitability: {:.6}", exploit);
    println!();

    // ── 2. Simulate all three strategies ─────────────────────────────────
    let mccfr_strat =
        AbstractedMcfrStrategy::new(tables[0].clone(), Box::new(BucketedAbstraction));

    println!("Simulating {} games per strategy...", num_games);
    let t0 = Instant::now();
    let random_results =
        simulate_commander_goldfish(&db, &deck, commander, &RandomStrategy, num_games);
    let greedy_results =
        simulate_commander_goldfish(&db, &deck, commander, &GreedyStrategy, num_games);
    let mccfr_results =
        simulate_commander_goldfish(&db, &deck, commander, &mccfr_strat, num_games);
    let sim_time = t0.elapsed();
    println!("Simulation complete in {:.1}s\n", sim_time.as_secs_f64());

    // ── 3. Strategy comparison ───────────────────────────────────────────
    println!("Strategy Comparison");
    println!("───────────────────");
    print_strategy_row("Random", &random_results);
    print_strategy_row("Greedy", &greedy_results);
    print_strategy_row("MCCFR", &mccfr_results);
    println!();

    // ── 4. Per-turn kill probability distribution ────────────────────────
    println!("Kill Turn Distribution (MCCFR)");
    println!("──────────────────────────────");
    print_kill_distribution(&mccfr_results);
    println!();

    // Side-by-side comparison
    println!("Kill Turn Distribution Comparison (cumulative P(win by turn X))");
    println!("───────────────────────────────────────────────────────────────");
    print_distribution_comparison(&random_results, &greedy_results, &mccfr_results);
    println!();

    // ── 5. Sample game trace ─────────────────────────────────────────────
    println!("Sample Game Trace (MCCFR trained strategy)");
    println!("──────────────────────────────────────────");
    println!("(Actions logged to stderr)\n");

    let result = run_commander_goldfish_game_verbose(&db, &deck, commander, &mccfr_strat);
    println!(
        "Result: {} in {} turns ({} actions), final life: {}/{}",
        match result.winner {
            Some(0) => "WIN",
            Some(_) => "LOSS",
            None => "DRAW",
        },
        result.turns,
        result.actions_taken,
        result.final_life[0],
        result.final_life[1],
    );
    println!();

    // ── 6. Policy snapshots at key decision points ───────────────────────
    println!("MCCFR Policy at Key Decision Points (pilot only)");
    println!("─────────────────────────────────────────────────");

    let mut replay_state = GameState::new_commander(2);
    replay_state.card_db = Some(Arc::new(db.clone()));
    rules::setup_commander_game(&mut replay_state, &deck, &deck, commander, commander);

    let snapshots = collect_policy_snapshots(&replay_state, &tables, &abstraction, 60);

    let mut shown = 0;
    for snap in &snapshots {
        // Only show pilot (P0) decisions
        if snap.player != 0 {
            continue;
        }
        shown += 1;
        println!(
            "Decision #{}: {} (visits: {})",
            shown, snap.state_description, snap.visit_count,
        );
        for (action_raw, prob) in &snap.action_distribution {
            if *prob > 0.01 {
                let action_pretty = resolve_card_names(action_raw, &card_names);
                println!("  {:5.1}%  {}", prob * 100.0, action_pretty);
            }
        }
        println!();
        if shown >= 20 {
            break;
        }
    }
}

/// Build a map of card_id -> card name for all cards in the deck.
fn build_card_names(db: &CardDatabase, deck: &[u64], commander: u64) -> HashMap<String, String> {
    let mut names = HashMap::new();
    for &card_id in deck {
        if let Some(def) = db.get(card_id) {
            names.insert(format!("{}", card_id), def.name.clone());
        }
    }
    if let Some(def) = db.get(commander) {
        names.insert(format!("{}", commander), def.name.clone());
    }
    names
}

/// Replace `card_id: NNN` patterns in an action string with card names.
fn resolve_card_names(action: &str, names: &HashMap<String, String>) -> String {
    let mut result = action.to_string();
    // Sort keys by length descending so "306" is replaced before "3"
    let mut keys: Vec<&String> = names.keys().collect();
    keys.sort_by(|a, b| b.len().cmp(&a.len()));

    for id_str in keys {
        let name = &names[id_str];

        // Match card_id followed by the number and a non-digit delimiter
        for delim in [",", "}", ")"] {
            let pattern = format!("card_id: {}{}", id_str, delim);
            let replacement = format!("card_id: {} ({}){}", id_str, name, delim);
            result = result.replace(&pattern, &replacement);
        }

        // Match attacker_card_ids tuples: (NNN, 0)
        // Only match when preceded by ( or ", " and followed by ", "
        let attack_pattern = format!("({}, ", id_str);
        let attack_replacement = format!("({} [{}], ", id_str, name);
        result = result.replace(&attack_pattern, &attack_replacement);
    }
    result
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

fn print_kill_distribution(r: &GoldfishResults) {
    if r.wins == 0 {
        println!("  No wins recorded.");
        return;
    }
    let mut cumulative = 0u64;
    println!("  Turn │ Wins │  P(kill) │ P(win by)");
    println!("  ─────┼──────┼──────────┼──────────");
    for (turn, &count) in r.kill_turn_distribution.iter().enumerate() {
        if count == 0 && cumulative == 0 {
            continue;
        }
        cumulative += count;
        if count > 0 || (cumulative > 0 && turn <= r.slowest_kill as usize) {
            let p_kill = count as f64 / r.total_games as f64;
            let p_cum = cumulative as f64 / r.total_games as f64;
            println!(
                "  T{:<3} │ {:>4} │  {:>5.1}%  │  {:>5.1}%",
                turn,
                count,
                p_kill * 100.0,
                p_cum * 100.0,
            );
        }
    }
    let no_win = r.total_games - r.wins;
    if no_win > 0 {
        println!(
            "  T20+ │ {:>4} │  {:>5.1}%  │   (did not win)",
            no_win,
            no_win as f64 / r.total_games as f64 * 100.0,
        );
    }
}

fn print_distribution_comparison(
    random: &GoldfishResults,
    greedy: &GoldfishResults,
    mccfr: &GoldfishResults,
) {
    println!("  Turn │   Random   │   Greedy   │    MCCFR");
    println!("  ─────┼────────────┼────────────┼────────────");

    let max_turn = [
        random.slowest_kill,
        greedy.slowest_kill,
        mccfr.slowest_kill,
        20,
    ]
    .into_iter()
    .max()
    .unwrap_or(20) as usize;

    let mut cum_r = 0u64;
    let mut cum_g = 0u64;
    let mut cum_m = 0u64;

    for turn in 1..=max_turn {
        let rc = random.kill_turn_distribution.get(turn).copied().unwrap_or(0);
        let gc = greedy.kill_turn_distribution.get(turn).copied().unwrap_or(0);
        let mc = mccfr.kill_turn_distribution.get(turn).copied().unwrap_or(0);
        cum_r += rc;
        cum_g += gc;
        cum_m += mc;

        if cum_r == 0 && cum_g == 0 && cum_m == 0 {
            continue;
        }

        let pr = cum_r as f64 / random.total_games as f64 * 100.0;
        let pg = cum_g as f64 / greedy.total_games as f64 * 100.0;
        let pm = cum_m as f64 / mccfr.total_games as f64 * 100.0;

        println!(
            "  T{:<3} │  {:>5.1}%     │  {:>5.1}%     │  {:>5.1}%",
            turn, pr, pg, pm,
        );
    }
}
