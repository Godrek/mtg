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
