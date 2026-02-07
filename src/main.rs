use mtg_gto::card::sample;
use mtg_gto::simulation::simulate;
use mtg_gto::strategy::{GreedyStrategy, RandomStrategy};

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
}
