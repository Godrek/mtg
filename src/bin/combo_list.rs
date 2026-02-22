//! Combo discovery tool — find all infinite combos in a deck.
//!
//! Takes a deck name and discovers all infinite combos among its cards
//! using exhaustive ability activation search.
//!
//! Usage:
//!   cargo run --release --bin combo_list
//!   DECK=kinnan cargo run --release --bin combo_list
//!   DECK=ashcoat MAX_PIECES=3 cargo run --release --bin combo_list
//!
//! Options (via environment variables):
//!   DECK=kinnan       Deck: "red", "green", "kinnan", "brimaz", "ashcoat", "thrun" (default: kinnan)
//!   MAX_PIECES=3      Max cards per combo (default: 3)
//!   MAX_DEPTH=20      DFS search depth (default: 20)
//!   MAX_MANA=10       Max startup mana to try (default: 10)
//!   MAX_CREATURES=5   Max startup creature tokens to try (default: 5)

use std::time::Instant;

use mtg_gto::card::sample;
use mtg_gto::combo::ComboCategory;
use mtg_gto::combo_discovery::{discover_combos, DiscoveryConfig};

fn main() {
    let deck_name = std::env::var("DECK").unwrap_or_else(|_| "kinnan".to_string());
    let max_pieces: usize = std::env::var("MAX_PIECES")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(3);
    let max_depth: usize = std::env::var("MAX_DEPTH")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(20);
    let max_mana: u32 = std::env::var("MAX_MANA")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(10);
    let max_creatures: u32 = std::env::var("MAX_CREATURES")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(5);

    let db = sample::build_sample_db();

    // Load deck
    let (deck_cards, deck_label) = match deck_name.as_str() {
        "red" => (sample::red_aggro_deck(), "Red Aggro"),
        "green" => (sample::green_stompy_deck(), "Green Stompy"),
        "brimaz" => {
            let (deck, commander) = sample::brimaz_commander_deck();
            let mut cards = deck;
            cards.push(commander);
            (cards, "Brimaz Commander")
        }
        "ashcoat" => {
            let (deck, commander) = sample::ashcoat_commander_deck();
            let mut cards = deck;
            cards.push(commander);
            (cards, "Ashcoat Commander")
        }
        "thrun" => {
            let (deck, commander) = sample::thrun_commander_deck();
            let mut cards = deck;
            cards.push(commander);
            (cards, "Thrun Commander")
        }
        _ => {
            let (deck, commander, _tutor_targets) = sample::kinnan_commander_deck();
            let mut cards = deck;
            cards.push(commander);
            (cards, "Kinnan Commander")
        }
    };

    // Deduplicate card IDs (deck has multiples of the same card)
    let mut unique_cards = deck_cards.clone();
    unique_cards.sort();
    unique_cards.dedup();

    let config = DiscoveryConfig {
        max_combo_pieces: max_pieces,
        max_depth,
        max_startup_mana: max_mana,
        max_startup_creatures: max_creatures,
        ..Default::default()
    };

    println!("Combo Discovery");
    println!("===============");
    println!("Deck:           {} ({} cards, {} unique)", deck_label, deck_cards.len(), unique_cards.len());
    println!("Max pieces:     {}", max_pieces);
    println!("Max DFS depth:  {}", max_depth);
    println!("Max startup:    {} mana, {} creatures", max_mana, max_creatures);
    println!();

    let t0 = Instant::now();
    let combos = discover_combos(&db, &unique_cards, &config);
    let elapsed = t0.elapsed();

    if combos.is_empty() {
        println!("No infinite combos found.");
    } else {
        println!("Found {} combo(s) in {:.2}s:\n", combos.len(), elapsed.as_secs_f64());

        // Group by category
        let mut by_category: std::collections::BTreeMap<String, Vec<usize>> =
            std::collections::BTreeMap::new();

        for (i, combo) in combos.iter().enumerate() {
            // Print full combo details
            println!("{}. {}", i + 1, combo);

            // Index by category
            for cat in &combo.categories {
                let cat_name = match cat {
                    ComboCategory::InfiniteMana => "Infinite Mana",
                    ComboCategory::InfiniteTokens => "Infinite Tokens",
                    ComboCategory::InfiniteDamage => "Infinite Damage",
                    ComboCategory::InfiniteLifeGain => "Infinite Life Gain",
                    ComboCategory::InfiniteDraw => "Infinite Draw",
                };
                by_category
                    .entry(cat_name.to_string())
                    .or_default()
                    .push(i + 1);
            }
        }

        // Print category summary
        println!("─────────────────────────────────");
        println!("Category Summary:");
        for (cat, indices) in &by_category {
            println!(
                "  {}: {} combo(s) [{}]",
                cat,
                indices.len(),
                indices
                    .iter()
                    .map(|i| format!("#{}", i))
                    .collect::<Vec<_>>()
                    .join(", ")
            );
        }
    }

    println!("\nCompleted in {:.2}s", elapsed.as_secs_f64());
}
