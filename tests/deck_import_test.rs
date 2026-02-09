use mtg_gto::card::sample;
use mtg_gto::deck_import::import_deck_from_file;
use std::fs;

#[test]
fn test_import_deck_from_file() {
    let db = sample::build_sample_db();
    let temp_dir = std::env::temp_dir();
    let deck_path = temp_dir.join("sample_deck.txt");
    let deck_contents = "\
2 Lightning Bolt
1 Mountain

1 Lightning Bolt
";
    fs::write(&deck_path, deck_contents).expect("write deck file");

    let deck = import_deck_from_file(&deck_path, &db).expect("import deck");

    assert_eq!(deck.name, "sample_deck");
    assert_eq!(deck.total_cards(), 4);
    let expanded = deck.expand();
    assert_eq!(expanded.len(), 4);
}

#[test]
fn test_import_commander_deck_sections() {
    let db = sample::build_sample_db();
    let temp_dir = std::env::temp_dir();
    let deck_path = temp_dir.join("commander_section_test.txt");
    let deck_contents = "\
~~Commanders~~
1 Kinnan, Bonder Prodigy

~~Mainboard~~
1 Sol Ring
1 Forest
1 Island
";
    fs::write(&deck_path, deck_contents).expect("write deck file");

    let deck = import_deck_from_file(&deck_path, &db).expect("import deck");

    // Commanders section should be parsed separately
    assert_eq!(deck.commanders.len(), 1);
    assert_eq!(deck.commanders[0].card_id, sample::ids::KINNAN_BONDER_PRODIGY);
    assert_eq!(deck.commanders[0].quantity, 1);

    // Mainboard cards
    assert_eq!(deck.total_cards(), 3);
    let expanded = deck.expand();
    assert_eq!(expanded.len(), 3);
}

#[test]
fn test_import_kinnan_deck_file() {
    let db = sample::build_sample_db();
    let deck_path = std::path::PathBuf::from("decks/kinnan_bonder_prodigy.txt");
    if !deck_path.exists() {
        return; // Skip if deck file not present
    }

    let deck = import_deck_from_file(&deck_path, &db).expect("import Kinnan deck");

    // Should have exactly 1 commander
    assert_eq!(deck.commanders.len(), 1);
    assert_eq!(deck.commanders[0].card_id, sample::ids::KINNAN_BONDER_PRODIGY);

    // Mainboard should have 99 cards (100 total - 1 commander)
    assert_eq!(deck.total_cards(), 99);
}
