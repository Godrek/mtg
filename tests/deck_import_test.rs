use mtg_gto::card::sample;
use mtg_gto::deck_import::{import_deck_from_file, DeckImportError};
use std::fs;
use std::path::PathBuf;

fn temp_deck_path(name: &str) -> PathBuf {
    let unique = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("time went backwards")
        .as_nanos();
    std::env::temp_dir().join(format!("{name}_{unique}.txt"))
}

#[test]
fn test_import_deck_from_file() {
    let db = sample::build_sample_db();
    let deck_path = temp_deck_path("sample_deck");
    let deck_contents = "\
2 Lightning Bolt
1 Mountain

1 Lightning Bolt
";
    fs::write(&deck_path, deck_contents).expect("write deck file");

    let deck = import_deck_from_file(&deck_path, &db).expect("import deck");

    assert!(deck.name.starts_with("sample_deck"));
    assert_eq!(deck.total_cards(), 4);
    let expanded = deck.expand();
    assert_eq!(expanded.len(), 4);

    assert_eq!(deck.cards.len(), 2);
    let lightning = deck
        .cards
        .iter()
        .find(|entry| entry.card_id == sample::ids::LIGHTNING_BOLT)
        .expect("missing lightning bolt");
    assert_eq!(lightning.quantity, 3);
}

#[test]
fn test_import_deck_from_file_comments_and_sideboard() {
    let db = sample::build_sample_db();
    let deck_path = temp_deck_path("comments_sideboard");
    let deck_contents = "\
# Main deck
2 Lightning Bolt
// lands
1 Mountain
Sideboard:
1 Island
";
    fs::write(&deck_path, deck_contents).expect("write deck file");

    let deck = import_deck_from_file(&deck_path, &db).expect("import deck");

    assert_eq!(deck.total_cards(), 3);
    assert_eq!(deck.cards.len(), 2);
}

#[test]
fn test_import_deck_from_file_invalid_quantity() {
    let db = sample::build_sample_db();
    let deck_path = temp_deck_path("invalid_quantity");
    let deck_contents = "\
0 Lightning Bolt
";
    fs::write(&deck_path, deck_contents).expect("write deck file");

    let err = import_deck_from_file(&deck_path, &db).expect_err("expected error");
    assert!(matches!(err, DeckImportError::InvalidLine { .. }));
}

#[test]
fn test_import_deck_from_file_unknown_card() {
    let db = sample::build_sample_db();
    let deck_path = temp_deck_path("unknown_card");
    let deck_contents = "\
1 Not A Real Card
";
    fs::write(&deck_path, deck_contents).expect("write deck file");

    let err = import_deck_from_file(&deck_path, &db).expect_err("expected error");
    assert!(matches!(err, DeckImportError::UnknownCard { .. }));
}

#[test]
fn test_import_deck_from_file_empty() {
    let db = sample::build_sample_db();
    let deck_path = temp_deck_path("empty_deck");
    fs::write(&deck_path, "").expect("write deck file");

    let deck = import_deck_from_file(&deck_path, &db).expect("import deck");
    assert_eq!(deck.total_cards(), 0);
    assert_eq!(deck.cards.len(), 0);
}

#[test]
fn test_import_deck_from_file_missing_file() {
    let db = sample::build_sample_db();
    let deck_path = temp_deck_path("missing_deck");
    let err = import_deck_from_file(&deck_path, &db).expect_err("expected error");
    assert!(matches!(err, DeckImportError::Io(_)));
}
