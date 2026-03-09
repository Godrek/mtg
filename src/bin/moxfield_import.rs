//! Moxfield deck importer binary.
//!
//! Fetches a deck from Moxfield, resolves all cards (sample DB first, then
//! Scryfall for unknowns), and saves the result into the `decks/` directory
//! so it can be used as a preset with `--preset <name>`.
//!
//! # Usage
//!
//! ```bash
//! cargo run --features scryfall --bin moxfield_import -- <URL-or-deck-ID>
//! cargo run --features scryfall --bin moxfield_import -- https://moxfield.com/decks/JHjwO92ZUEyNdPzE7D5d7A
//! cargo run --features scryfall --bin moxfield_import -- JHjwO92ZUEyNdPzE7D5d7A
//! ```
//!
//! After importing, run a goldfish simulation:
//! ```bash
//! cargo run --release --bin goldfish -- --preset <name>
//! ```

use mtg_gto::moxfield;

fn main() {
    let args: Vec<String> = std::env::args().collect();

    let url_or_id = match args.get(1) {
        Some(s) => s.as_str(),
        None => {
            eprintln!("Usage: moxfield_import <moxfield-url-or-deck-id>");
            eprintln!();
            eprintln!("Examples:");
            eprintln!(
                "  moxfield_import https://moxfield.com/decks/JHjwO92ZUEyNdPzE7D5d7A"
            );
            eprintln!("  moxfield_import JHjwO92ZUEyNdPzE7D5d7A");
            std::process::exit(1);
        }
    };

    eprintln!("Moxfield Deck Importer");
    eprintln!("======================");

    match moxfield::import_moxfield_deck(url_or_id, "decks", ".scryfall_cache") {
        Ok(result) => {
            println!();
            println!("Import complete!");
            println!("  Deck name : {}", result.deck_name);
            println!("  Cards     : {}", result.deck.len());
            if let Some(cmd_id) = result.commander_id {
                if let Some(def) = result.db.get(cmd_id) {
                    println!("  Commander : {}", def.name);
                }
            }
            println!("  Deck file : {}", result.deck_txt_path);
            if let Some(ref json_path) = result.cards_json_path {
                println!("  Card defs : {json_path}");
            }

            if !result.errors.is_empty() {
                println!();
                println!("Warnings ({} card(s) could not be fetched):", result.errors.len());
                for err in &result.errors {
                    println!("  - {err}");
                }
            }

            println!();
            println!("To simulate this deck, run:");
            println!(
                "  cargo run --release --bin goldfish -- --preset {}",
                result.file_stem
            );
        }
        Err(e) => {
            eprintln!("Error: {e}");
            std::process::exit(1);
        }
    }
}
