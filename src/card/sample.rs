//! Sample card definitions for testing — classic MTG cards.

use crate::card::*;
use crate::game::CardDatabase;
use crate::mana::{Color, ManaCost};

/// Card IDs for sample cards.
pub mod ids {
    pub const MOUNTAIN: u64 = 1;
    pub const FOREST: u64 = 2;
    pub const PLAINS: u64 = 3;
    pub const ISLAND: u64 = 4;
    pub const SWAMP: u64 = 5;

    pub const LIGHTNING_BOLT: u64 = 100;
    pub const GIANT_GROWTH: u64 = 101;
    pub const GRIZZLY_BEARS: u64 = 102;
    pub const GREY_OGRE: u64 = 103;
    pub const SERRA_ANGEL: u64 = 104;
    pub const SHIVAN_DRAGON: u64 = 105;
    pub const LLANOWAR_ELVES: u64 = 106;
    pub const SWORDS_TO_PLOWSHARES: u64 = 107;
    pub const COUNTERSPELL: u64 = 108;
    pub const DARK_RITUAL: u64 = 109;
    pub const SAVANNAH_LIONS: u64 = 110;
    pub const SHOCK: u64 = 111;
    pub const GOBLIN_GUIDE: u64 = 112;
    pub const ELVISH_MYSTIC: u64 = 113;
    pub const KALONIAN_TUSKER: u64 = 114;
    pub const LEATHERBACK_BALOTH: u64 = 115;
    pub const MONASTERY_SWIFTSPEAR: u64 = 116;
    pub const LAVA_SPIKE: u64 = 117;
    pub const RIFT_BOLT: u64 = 118;
    pub const STOMPING_GROUND: u64 = 119; // dual land placeholder
    pub const ELVISH_VISIONARY: u64 = 120;
    pub const BLADE_SPLICER: u64 = 121;
    pub const SIEGE_GANG_COMMANDER: u64 = 122;
}

pub fn build_sample_db() -> CardDatabase {
    let mut db = CardDatabase::new();

    // === Basic Lands ===
    db.insert(CardDef {
        id: ids::MOUNTAIN,
        name: "Mountain".into(),
        mana_cost: None,
        card_types: vec![CardType::Land],
        supertypes: vec![Supertype::Basic],
        subtypes: vec![Subtype("Mountain".into())],
        keywords: vec![],
        power: None,
        toughness: None,
        mana_abilities: vec![ManaAbility::TapForColor(Color::Red)],
        spell_effect: None,
        activated_abilities: vec![],
        triggered_abilities: vec![],
        starting_loyalty: None,
        enters_tapped: false,
        oracle_text: "{T}: Add {R}.".into(),
    });

    db.insert(CardDef {
        id: ids::FOREST,
        name: "Forest".into(),
        mana_cost: None,
        card_types: vec![CardType::Land],
        supertypes: vec![Supertype::Basic],
        subtypes: vec![Subtype("Forest".into())],
        keywords: vec![],
        power: None,
        toughness: None,
        mana_abilities: vec![ManaAbility::TapForColor(Color::Green)],
        spell_effect: None,
        activated_abilities: vec![],
        triggered_abilities: vec![],
        starting_loyalty: None,
        enters_tapped: false,
        oracle_text: "{T}: Add {G}.".into(),
    });

    db.insert(CardDef {
        id: ids::PLAINS,
        name: "Plains".into(),
        mana_cost: None,
        card_types: vec![CardType::Land],
        supertypes: vec![Supertype::Basic],
        subtypes: vec![Subtype("Plains".into())],
        keywords: vec![],
        power: None,
        toughness: None,
        mana_abilities: vec![ManaAbility::TapForColor(Color::White)],
        spell_effect: None,
        activated_abilities: vec![],
        triggered_abilities: vec![],
        starting_loyalty: None,
        enters_tapped: false,
        oracle_text: "{T}: Add {W}.".into(),
    });

    db.insert(CardDef {
        id: ids::ISLAND,
        name: "Island".into(),
        mana_cost: None,
        card_types: vec![CardType::Land],
        supertypes: vec![Supertype::Basic],
        subtypes: vec![Subtype("Island".into())],
        keywords: vec![],
        power: None,
        toughness: None,
        mana_abilities: vec![ManaAbility::TapForColor(Color::Blue)],
        spell_effect: None,
        activated_abilities: vec![],
        triggered_abilities: vec![],
        starting_loyalty: None,
        enters_tapped: false,
        oracle_text: "{T}: Add {U}.".into(),
    });

    db.insert(CardDef {
        id: ids::SWAMP,
        name: "Swamp".into(),
        mana_cost: None,
        card_types: vec![CardType::Land],
        supertypes: vec![Supertype::Basic],
        subtypes: vec![Subtype("Swamp".into())],
        keywords: vec![],
        power: None,
        toughness: None,
        mana_abilities: vec![ManaAbility::TapForColor(Color::Black)],
        spell_effect: None,
        activated_abilities: vec![],
        triggered_abilities: vec![],
        starting_loyalty: None,
        enters_tapped: false,
        oracle_text: "{T}: Add {B}.".into(),
    });

    // === Red Creatures ===
    db.insert(CardDef {
        id: ids::GREY_OGRE,
        name: "Gray Ogre".into(),
        mana_cost: Some(ManaCost::new(2, 0, 0, 0, 1, 0)),
        card_types: vec![CardType::Creature],
        supertypes: vec![],
        subtypes: vec![Subtype("Ogre".into())],
        keywords: vec![],
        power: Some(2),
        toughness: Some(2),
        mana_abilities: vec![],
        spell_effect: None,
        activated_abilities: vec![],
        triggered_abilities: vec![],
        starting_loyalty: None,
        enters_tapped: false,
        oracle_text: "".into(),
    });

    db.insert(CardDef {
        id: ids::GOBLIN_GUIDE,
        name: "Goblin Guide".into(),
        mana_cost: Some(ManaCost::new(0, 0, 0, 0, 1, 0)),
        card_types: vec![CardType::Creature],
        supertypes: vec![],
        subtypes: vec![Subtype("Goblin".into()), Subtype("Scout".into())],
        keywords: vec![KeywordAbility::Haste],
        power: Some(2),
        toughness: Some(2),
        mana_abilities: vec![],
        spell_effect: None,
        activated_abilities: vec![],
        triggered_abilities: vec![], // simplified — no "reveal top" trigger
        starting_loyalty: None,
        enters_tapped: false,
        oracle_text: "Haste".into(),
    });

    db.insert(CardDef {
        id: ids::MONASTERY_SWIFTSPEAR,
        name: "Monastery Swiftspear".into(),
        mana_cost: Some(ManaCost::new(0, 0, 0, 0, 1, 0)),
        card_types: vec![CardType::Creature],
        supertypes: vec![],
        subtypes: vec![Subtype("Human".into()), Subtype("Monk".into())],
        keywords: vec![KeywordAbility::Haste],
        power: Some(1),
        toughness: Some(2),
        mana_abilities: vec![],
        spell_effect: None,
        activated_abilities: vec![],
        triggered_abilities: vec![], // simplified — no prowess
        starting_loyalty: None,
        enters_tapped: false,
        oracle_text: "Haste, Prowess".into(),
    });

    db.insert(CardDef {
        id: ids::SHIVAN_DRAGON,
        name: "Shivan Dragon".into(),
        mana_cost: Some(ManaCost::new(4, 0, 0, 0, 2, 0)),
        card_types: vec![CardType::Creature],
        supertypes: vec![],
        subtypes: vec![Subtype("Dragon".into())],
        keywords: vec![KeywordAbility::Flying],
        power: Some(5),
        toughness: Some(5),
        mana_abilities: vec![],
        spell_effect: None,
        activated_abilities: vec![], // simplified — no firebreathing
        triggered_abilities: vec![],
        starting_loyalty: None,
        enters_tapped: false,
        oracle_text: "Flying. {R}: Shivan Dragon gets +1/+0 until end of turn.".into(),
    });

    // === Green Creatures ===
    db.insert(CardDef {
        id: ids::GRIZZLY_BEARS,
        name: "Grizzly Bears".into(),
        mana_cost: Some(ManaCost::new(1, 0, 0, 0, 0, 1)),
        card_types: vec![CardType::Creature],
        supertypes: vec![],
        subtypes: vec![Subtype("Bear".into())],
        keywords: vec![],
        power: Some(2),
        toughness: Some(2),
        mana_abilities: vec![],
        spell_effect: None,
        activated_abilities: vec![],
        triggered_abilities: vec![],
        starting_loyalty: None,
        enters_tapped: false,
        oracle_text: "".into(),
    });

    db.insert(CardDef {
        id: ids::LLANOWAR_ELVES,
        name: "Llanowar Elves".into(),
        mana_cost: Some(ManaCost::new(0, 0, 0, 0, 0, 1)),
        card_types: vec![CardType::Creature],
        supertypes: vec![],
        subtypes: vec![Subtype("Elf".into()), Subtype("Druid".into())],
        keywords: vec![],
        power: Some(1),
        toughness: Some(1),
        mana_abilities: vec![ManaAbility::TapForColor(Color::Green)],
        spell_effect: None,
        activated_abilities: vec![],
        triggered_abilities: vec![],
        starting_loyalty: None,
        enters_tapped: false,
        oracle_text: "{T}: Add {G}.".into(),
    });

    db.insert(CardDef {
        id: ids::ELVISH_MYSTIC,
        name: "Elvish Mystic".into(),
        mana_cost: Some(ManaCost::new(0, 0, 0, 0, 0, 1)),
        card_types: vec![CardType::Creature],
        supertypes: vec![],
        subtypes: vec![Subtype("Elf".into()), Subtype("Druid".into())],
        keywords: vec![],
        power: Some(1),
        toughness: Some(1),
        mana_abilities: vec![ManaAbility::TapForColor(Color::Green)],
        spell_effect: None,
        activated_abilities: vec![],
        triggered_abilities: vec![],
        starting_loyalty: None,
        enters_tapped: false,
        oracle_text: "{T}: Add {G}.".into(),
    });

    db.insert(CardDef {
        id: ids::KALONIAN_TUSKER,
        name: "Kalonian Tusker".into(),
        mana_cost: Some(ManaCost::new(0, 0, 0, 0, 0, 2)),
        card_types: vec![CardType::Creature],
        supertypes: vec![],
        subtypes: vec![Subtype("Beast".into())],
        keywords: vec![],
        power: Some(3),
        toughness: Some(3),
        mana_abilities: vec![],
        spell_effect: None,
        activated_abilities: vec![],
        triggered_abilities: vec![],
        starting_loyalty: None,
        enters_tapped: false,
        oracle_text: "".into(),
    });

    db.insert(CardDef {
        id: ids::LEATHERBACK_BALOTH,
        name: "Leatherback Baloth".into(),
        mana_cost: Some(ManaCost::new(0, 0, 0, 0, 0, 3)),
        card_types: vec![CardType::Creature],
        supertypes: vec![],
        subtypes: vec![Subtype("Beast".into())],
        keywords: vec![],
        power: Some(4),
        toughness: Some(5),
        mana_abilities: vec![],
        spell_effect: None,
        activated_abilities: vec![],
        triggered_abilities: vec![],
        starting_loyalty: None,
        enters_tapped: false,
        oracle_text: "".into(),
    });

    // === White Creatures ===
    db.insert(CardDef {
        id: ids::SAVANNAH_LIONS,
        name: "Savannah Lions".into(),
        mana_cost: Some(ManaCost::new(0, 1, 0, 0, 0, 0)),
        card_types: vec![CardType::Creature],
        supertypes: vec![],
        subtypes: vec![Subtype("Cat".into())],
        keywords: vec![],
        power: Some(2),
        toughness: Some(1),
        mana_abilities: vec![],
        spell_effect: None,
        activated_abilities: vec![],
        triggered_abilities: vec![],
        starting_loyalty: None,
        enters_tapped: false,
        oracle_text: "".into(),
    });

    db.insert(CardDef {
        id: ids::SERRA_ANGEL,
        name: "Serra Angel".into(),
        mana_cost: Some(ManaCost::new(3, 2, 0, 0, 0, 0)),
        card_types: vec![CardType::Creature],
        supertypes: vec![],
        subtypes: vec![Subtype("Angel".into())],
        keywords: vec![KeywordAbility::Flying, KeywordAbility::Vigilance],
        power: Some(4),
        toughness: Some(4),
        mana_abilities: vec![],
        spell_effect: None,
        activated_abilities: vec![],
        triggered_abilities: vec![],
        starting_loyalty: None,
        enters_tapped: false,
        oracle_text: "Flying, Vigilance".into(),
    });

    // === Instants / Sorceries ===
    db.insert(CardDef {
        id: ids::LIGHTNING_BOLT,
        name: "Lightning Bolt".into(),
        mana_cost: Some(ManaCost::new(0, 0, 0, 0, 1, 0)),
        card_types: vec![CardType::Instant],
        supertypes: vec![],
        subtypes: vec![],
        keywords: vec![],
        power: None,
        toughness: None,
        mana_abilities: vec![],
        spell_effect: Some(Effect::DealDamage {
            amount: 3,
            target: TargetSpec::CreatureOrPlayer,
        }),
        activated_abilities: vec![],
        triggered_abilities: vec![],
        starting_loyalty: None,
        enters_tapped: false,
        oracle_text: "Lightning Bolt deals 3 damage to any target.".into(),
    });

    db.insert(CardDef {
        id: ids::SHOCK,
        name: "Shock".into(),
        mana_cost: Some(ManaCost::new(0, 0, 0, 0, 1, 0)),
        card_types: vec![CardType::Instant],
        supertypes: vec![],
        subtypes: vec![],
        keywords: vec![],
        power: None,
        toughness: None,
        mana_abilities: vec![],
        spell_effect: Some(Effect::DealDamage {
            amount: 2,
            target: TargetSpec::CreatureOrPlayer,
        }),
        activated_abilities: vec![],
        triggered_abilities: vec![],
        starting_loyalty: None,
        enters_tapped: false,
        oracle_text: "Shock deals 2 damage to any target.".into(),
    });

    db.insert(CardDef {
        id: ids::LAVA_SPIKE,
        name: "Lava Spike".into(),
        mana_cost: Some(ManaCost::new(0, 0, 0, 0, 1, 0)),
        card_types: vec![CardType::Sorcery],
        supertypes: vec![],
        subtypes: vec![Subtype("Arcane".into())],
        keywords: vec![],
        power: None,
        toughness: None,
        mana_abilities: vec![],
        spell_effect: Some(Effect::DealDamage {
            amount: 3,
            target: TargetSpec::AnyPlayer,
        }),
        activated_abilities: vec![],
        triggered_abilities: vec![],
        starting_loyalty: None,
        enters_tapped: false,
        oracle_text: "Lava Spike deals 3 damage to target player or planeswalker.".into(),
    });

    db.insert(CardDef {
        id: ids::RIFT_BOLT,
        name: "Rift Bolt".into(),
        mana_cost: Some(ManaCost::new(2, 0, 0, 0, 1, 0)),
        card_types: vec![CardType::Sorcery],
        supertypes: vec![],
        subtypes: vec![],
        keywords: vec![],
        power: None,
        toughness: None,
        mana_abilities: vec![],
        spell_effect: Some(Effect::DealDamage {
            amount: 3,
            target: TargetSpec::CreatureOrPlayer,
        }),
        activated_abilities: vec![],
        triggered_abilities: vec![],
        starting_loyalty: None,
        enters_tapped: false,
        oracle_text: "Rift Bolt deals 3 damage to any target.".into(),
    });

    db.insert(CardDef {
        id: ids::GIANT_GROWTH,
        name: "Giant Growth".into(),
        mana_cost: Some(ManaCost::new(0, 0, 0, 0, 0, 1)),
        card_types: vec![CardType::Instant],
        supertypes: vec![],
        subtypes: vec![],
        keywords: vec![],
        power: None,
        toughness: None,
        mana_abilities: vec![],
        spell_effect: Some(Effect::Buff {
            power: 3,
            toughness: 3,
            until_eot: true,
        }),
        activated_abilities: vec![],
        triggered_abilities: vec![],
        starting_loyalty: None,
        enters_tapped: false,
        oracle_text: "Target creature gets +3/+3 until end of turn.".into(),
    });

    db.insert(CardDef {
        id: ids::SWORDS_TO_PLOWSHARES,
        name: "Swords to Plowshares".into(),
        mana_cost: Some(ManaCost::new(0, 1, 0, 0, 0, 0)),
        card_types: vec![CardType::Instant],
        supertypes: vec![],
        subtypes: vec![],
        keywords: vec![],
        power: None,
        toughness: None,
        mana_abilities: vec![],
        spell_effect: Some(Effect::DestroyTarget {
            target: TargetSpec::AnyCreature,
        }),
        // Simplified — real StP exiles and gains life
        activated_abilities: vec![],
        triggered_abilities: vec![],
        starting_loyalty: None,
        enters_tapped: false,
        oracle_text: "Exile target creature. Its controller gains life equal to its power.".into(),
    });

    db.insert(CardDef {
        id: ids::COUNTERSPELL,
        name: "Counterspell".into(),
        mana_cost: Some(ManaCost::new(0, 0, 2, 0, 0, 0)),
        card_types: vec![CardType::Instant],
        supertypes: vec![],
        subtypes: vec![],
        keywords: vec![],
        power: None,
        toughness: None,
        mana_abilities: vec![],
        spell_effect: Some(Effect::Counter {
            target: TargetSpec::AnySpell,
        }),
        activated_abilities: vec![],
        triggered_abilities: vec![],
        starting_loyalty: None,
        enters_tapped: false,
        oracle_text: "Counter target spell.".into(),
    });

    // === Cards with Triggered Abilities ===

    // Elvish Visionary: 1G 1/1 Elf Shaman — ETB draw a card
    db.insert(CardDef {
        id: ids::ELVISH_VISIONARY,
        name: "Elvish Visionary".into(),
        mana_cost: Some(ManaCost::new(1, 0, 0, 0, 0, 1)),
        card_types: vec![CardType::Creature],
        supertypes: vec![],
        subtypes: vec![Subtype("Elf".into()), Subtype("Shaman".into())],
        keywords: vec![],
        power: Some(1),
        toughness: Some(1),
        mana_abilities: vec![],
        spell_effect: None,
        activated_abilities: vec![],
        triggered_abilities: vec![TriggeredAbility {
            trigger: TriggerCondition::EntersBattlefield,
            effect: Effect::DrawCards { count: 1 },
            description: "When Elvish Visionary enters the battlefield, draw a card.".into(),
        }],
        starting_loyalty: None,
        enters_tapped: false,
        oracle_text: "When Elvish Visionary enters the battlefield, draw a card.".into(),
    });

    // Blade Splicer: 2W 1/1 Human Artificer — ETB create a 3/3 Golem with first strike
    db.insert(CardDef {
        id: ids::BLADE_SPLICER,
        name: "Blade Splicer".into(),
        mana_cost: Some(ManaCost::new(2, 1, 0, 0, 0, 0)),
        card_types: vec![CardType::Creature],
        supertypes: vec![],
        subtypes: vec![Subtype("Human".into()), Subtype("Artificer".into())],
        keywords: vec![],
        power: Some(1),
        toughness: Some(1),
        mana_abilities: vec![],
        spell_effect: None,
        activated_abilities: vec![],
        triggered_abilities: vec![TriggeredAbility {
            trigger: TriggerCondition::EntersBattlefield,
            effect: Effect::CreateToken(TokenDef {
                name: "Phyrexian Golem".into(),
                power: 3,
                toughness: 3,
                colors: vec![],
                subtypes: vec![Subtype("Phyrexian".into()), Subtype("Golem".into())],
                keywords: vec![KeywordAbility::FirstStrike],
            }),
            description: "When Blade Splicer enters the battlefield, create a 3/3 colorless Phyrexian Golem artifact creature token with first strike.".into(),
        }],
        starting_loyalty: None,
        enters_tapped: false,
        oracle_text: "When Blade Splicer enters the battlefield, create a 3/3 colorless Phyrexian Golem artifact creature token with first strike.".into(),
    });

    // Siege-Gang Commander: 3RR 2/2 Goblin — ETB create three 1/1 Goblin tokens
    db.insert(CardDef {
        id: ids::SIEGE_GANG_COMMANDER,
        name: "Siege-Gang Commander".into(),
        mana_cost: Some(ManaCost::new(3, 0, 0, 0, 2, 0)),
        card_types: vec![CardType::Creature],
        supertypes: vec![],
        subtypes: vec![Subtype("Goblin".into())],
        keywords: vec![],
        power: Some(2),
        toughness: Some(2),
        mana_abilities: vec![],
        spell_effect: None,
        activated_abilities: vec![],
        triggered_abilities: vec![TriggeredAbility {
            trigger: TriggerCondition::EntersBattlefield,
            effect: Effect::Multiple(vec![
                Effect::CreateToken(TokenDef {
                    name: "Goblin".into(),
                    power: 1,
                    toughness: 1,
                    colors: vec![Color::Red],
                    subtypes: vec![Subtype("Goblin".into())],
                    keywords: vec![],
                }),
                Effect::CreateToken(TokenDef {
                    name: "Goblin".into(),
                    power: 1,
                    toughness: 1,
                    colors: vec![Color::Red],
                    subtypes: vec![Subtype("Goblin".into())],
                    keywords: vec![],
                }),
                Effect::CreateToken(TokenDef {
                    name: "Goblin".into(),
                    power: 1,
                    toughness: 1,
                    colors: vec![Color::Red],
                    subtypes: vec![Subtype("Goblin".into())],
                    keywords: vec![],
                }),
            ]),
            description: "When Siege-Gang Commander enters the battlefield, create three 1/1 red Goblin creature tokens.".into(),
        }],
        starting_loyalty: None,
        enters_tapped: false,
        oracle_text: "When Siege-Gang Commander enters the battlefield, create three 1/1 red Goblin creature tokens.".into(),
    });

    db
}

/// Build a minimal 15-card mono-red burn deck for MCCFR training.
///
/// Phase 1B training scenario: Mountains + Lightning Bolts.
/// Games end in 3-5 turns. Reachable info sets fit in memory (<100K entries).
pub fn mini_red_burn() -> Vec<CardId> {
    let mut deck = Vec::new();
    // 8 Mountains
    for _ in 0..8 {
        deck.push(ids::MOUNTAIN);
    }
    // 4 Lightning Bolt
    for _ in 0..4 {
        deck.push(ids::LIGHTNING_BOLT);
    }
    // 3 Shock (additional burn)
    for _ in 0..3 {
        deck.push(ids::SHOCK);
    }
    assert_eq!(deck.len(), 15);
    deck
}

/// Build a minimal 15-card mono-red creature deck for MCCFR training.
///
/// Phase 1B training scenario: Mountains + Grey Ogres.
/// Simpler creature-based strategy against the burn deck.
pub fn mini_red_creatures() -> Vec<CardId> {
    let mut deck = Vec::new();
    // 8 Mountains
    for _ in 0..8 {
        deck.push(ids::MOUNTAIN);
    }
    // 4 Grey Ogre (2/2 for 2R)
    for _ in 0..4 {
        deck.push(ids::GREY_OGRE);
    }
    // 3 Goblin Guide (2/2 haste for R)
    for _ in 0..3 {
        deck.push(ids::GOBLIN_GUIDE);
    }
    assert_eq!(deck.len(), 15);
    deck
}

/// Build a mono-red aggro decklist (burn).
pub fn red_aggro_deck() -> Vec<CardId> {
    let mut deck = Vec::new();

    // 20 Mountains
    for _ in 0..20 {
        deck.push(ids::MOUNTAIN);
    }

    // 4 Goblin Guide
    for _ in 0..4 {
        deck.push(ids::GOBLIN_GUIDE);
    }

    // 4 Monastery Swiftspear
    for _ in 0..4 {
        deck.push(ids::MONASTERY_SWIFTSPEAR);
    }

    // 4 Lightning Bolt
    for _ in 0..4 {
        deck.push(ids::LIGHTNING_BOLT);
    }

    // 4 Lava Spike
    for _ in 0..4 {
        deck.push(ids::LAVA_SPIKE);
    }

    // 4 Shock
    for _ in 0..4 {
        deck.push(ids::SHOCK);
    }

    // 4 Rift Bolt
    for _ in 0..4 {
        deck.push(ids::RIFT_BOLT);
    }

    // 4 Grey Ogre (filler 3-drop creature)
    for _ in 0..4 {
        deck.push(ids::GREY_OGRE);
    }

    // 12 more Mountains to hit 60
    for _ in 0..12 {
        deck.push(ids::MOUNTAIN);
    }

    assert_eq!(deck.len(), 60);
    deck
}

/// Build a mono-green stompy decklist.
pub fn green_stompy_deck() -> Vec<CardId> {
    let mut deck = Vec::new();

    // 20 Forests
    for _ in 0..20 {
        deck.push(ids::FOREST);
    }

    // 4 Llanowar Elves
    for _ in 0..4 {
        deck.push(ids::LLANOWAR_ELVES);
    }

    // 4 Elvish Mystic
    for _ in 0..4 {
        deck.push(ids::ELVISH_MYSTIC);
    }

    // 4 Kalonian Tusker
    for _ in 0..4 {
        deck.push(ids::KALONIAN_TUSKER);
    }

    // 4 Grizzly Bears
    for _ in 0..4 {
        deck.push(ids::GRIZZLY_BEARS);
    }

    // 4 Leatherback Baloth
    for _ in 0..4 {
        deck.push(ids::LEATHERBACK_BALOTH);
    }

    // 4 Giant Growth
    for _ in 0..4 {
        deck.push(ids::GIANT_GROWTH);
    }

    // 8 more Forests + 8 more Bears to hit 60
    for _ in 0..8 {
        deck.push(ids::FOREST);
    }
    for _ in 0..8 {
        deck.push(ids::GRIZZLY_BEARS);
    }

    assert_eq!(deck.len(), 60);
    deck
}
