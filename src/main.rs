use std::sync::Arc;

use mtg_gto::card::sample;
use mtg_gto::game::GameState;
use mtg_gto::rules;
use mtg_gto::simulation::{simulate, simulate_goldfish};
use mtg_gto::solver::mccfr::{self, McfrConfig};
use mtg_gto::strategy::{GreedyStrategy, McfrStrategy, RandomStrategy};

fn main() {
    println!("MTG GTO Simulator");
    println!("==================\n");

    // Build card database
    let db = sample::build_sample_db();

    // Build decks
    let red_deck = sample::red_aggro_deck();
    let green_deck = sample::green_stompy_deck();

    // Run simulations
    let greedy = GreedyStrategy;
    let random = RandomStrategy;

    println!("Red Aggro (Greedy) vs Green Stompy (Greedy):");
    let results = simulate(&db, &red_deck, &green_deck, &greedy, &greedy, 1000);
    results.display();

    println!();

    println!("Red Aggro (Greedy) vs Green Stompy (Random):");
    let results = simulate(&db, &red_deck, &green_deck, &greedy, &random, 1000);
    results.display();

    println!();

    println!("Red Aggro (Random) vs Green Stompy (Greedy):");
    let results = simulate(&db, &red_deck, &green_deck, &random, &greedy, 1000);
    results.display();

    // --- Goldfish mode ---
    println!("\n==================");
    println!("Goldfish Mode");
    println!("==================\n");

    println!("Red Aggro (Greedy) — Goldfish:");
    let greedy_red = simulate_goldfish(&db, &red_deck, &greedy, 1000);
    greedy_red.display();

    println!();

    println!("Green Stompy (Greedy) — Goldfish:");
    let greedy_green = simulate_goldfish(&db, &green_deck, &greedy, 1000);
    greedy_green.display();

    // --- MCCFR Goldfish ---
    println!("\n==================");
    println!("MCCFR Goldfish Training");
    println!("==================\n");

    let config = McfrConfig { max_depth: 8, max_actions: 1000 };

    // Train MCCFR for Red Aggro goldfish
    println!("Training MCCFR for Red Aggro goldfish (50 iterations)...");
    let mut red_state = GameState::new(2);
    red_state.card_db = Some(Arc::new(db.clone()));
    rules::setup_game(&mut red_state, &red_deck, &red_deck);
    let red_tables = mccfr::train_goldfish(&red_state, 50, &config);

    let stats = mccfr::training_stats(&red_tables);
    println!(
        "  Trained: {} info sets, {} visits",
        stats.total_info_sets[0], stats.total_visits[0],
    );

    let mccfr_red = McfrStrategy::new(red_tables[0].clone());
    println!("\nRed Aggro (MCCFR) — Goldfish:");
    let mccfr_red_results = simulate_goldfish(&db, &red_deck, &mccfr_red, 1000);
    mccfr_red_results.display();

    println!("\n  >> Greedy avg kill: T{:.2}  |  MCCFR avg kill: T{:.2}  |  delta: {:.2} turns",
        greedy_red.avg_kill_turn,
        mccfr_red_results.avg_kill_turn,
        greedy_red.avg_kill_turn - mccfr_red_results.avg_kill_turn,
    );

    // Train MCCFR for Green Stompy goldfish
    println!("\nTraining MCCFR for Green Stompy goldfish (50 iterations)...");
    let mut green_state = GameState::new(2);
    green_state.card_db = Some(Arc::new(db.clone()));
    rules::setup_game(&mut green_state, &green_deck, &green_deck);
    let green_tables = mccfr::train_goldfish(&green_state, 50, &config);

    let stats = mccfr::training_stats(&green_tables);
    println!(
        "  Trained: {} info sets, {} visits",
        stats.total_info_sets[0], stats.total_visits[0],
    );

    let mccfr_green = McfrStrategy::new(green_tables[0].clone());
    println!("\nGreen Stompy (MCCFR) — Goldfish:");
    let mccfr_green_results = simulate_goldfish(&db, &green_deck, &mccfr_green, 1000);
    mccfr_green_results.display();

    println!("\n  >> Greedy avg kill: T{:.2}  |  MCCFR avg kill: T{:.2}  |  delta: {:.2} turns",
        greedy_green.avg_kill_turn,
        mccfr_green_results.avg_kill_turn,
        greedy_green.avg_kill_turn - mccfr_green_results.avg_kill_turn,
    );
}
