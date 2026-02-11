//! Sample card definitions for testing — classic MTG cards.

use crate::card::*;
use crate::game::CardDatabase;
use crate::layers::{AffectedObjects, StaticAbility};
use crate::mana::{Color, ManaCost};

/// Card IDs for sample cards.
pub mod ids {
    pub use crate::card::catalog::ids::*;
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
        static_abilities: vec![],
        enters_tapped: false,
        oracle_text: "{T}: Add {R}.".into(),
        ..Default::default()
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
        static_abilities: vec![],
        enters_tapped: false,
        oracle_text: "{T}: Add {G}.".into(),
        ..Default::default()
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
        static_abilities: vec![],
        enters_tapped: false,
        oracle_text: "{T}: Add {W}.".into(),
        ..Default::default()
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
        static_abilities: vec![],
        enters_tapped: false,
        oracle_text: "{T}: Add {U}.".into(),
        ..Default::default()
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
        static_abilities: vec![],
        enters_tapped: false,
        oracle_text: "{T}: Add {B}.".into(),
        ..Default::default()
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
        static_abilities: vec![],
        enters_tapped: false,
        oracle_text: "".into(),
        ..Default::default()
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
        static_abilities: vec![],
        enters_tapped: false,
        oracle_text: "Haste".into(),
        ..Default::default()
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
        static_abilities: vec![],
        enters_tapped: false,
        oracle_text: "Haste, Prowess".into(),
        ..Default::default()
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
        activated_abilities: vec![ActivatedAbility {
            cost: ManaCost::new(0, 0, 0, 0, 1, 0),
            requires_tap: false,
            effect: Effect::Buff {
                power: 1,
                toughness: 0,
                until_eot: true,
            },
            description: "{R}: Shivan Dragon gets +1/+0 until end of turn.".into(),
        }],
        triggered_abilities: vec![],
        starting_loyalty: None,
        static_abilities: vec![],
        enters_tapped: false,
        oracle_text: "Flying. {R}: Shivan Dragon gets +1/+0 until end of turn.".into(),
        ..Default::default()
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
        static_abilities: vec![],
        enters_tapped: false,
        oracle_text: "".into(),
        ..Default::default()
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
        static_abilities: vec![],
        enters_tapped: false,
        oracle_text: "{T}: Add {G}.".into(),
        ..Default::default()
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
        static_abilities: vec![],
        enters_tapped: false,
        oracle_text: "{T}: Add {G}.".into(),
        ..Default::default()
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
        static_abilities: vec![],
        enters_tapped: false,
        oracle_text: "".into(),
        ..Default::default()
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
        static_abilities: vec![],
        enters_tapped: false,
        oracle_text: "".into(),
        ..Default::default()
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
        static_abilities: vec![],
        enters_tapped: false,
        oracle_text: "".into(),
        ..Default::default()
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
        static_abilities: vec![],
        enters_tapped: false,
        oracle_text: "Flying, Vigilance".into(),
        ..Default::default()
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
        static_abilities: vec![],
        enters_tapped: false,
        oracle_text: "Lightning Bolt deals 3 damage to any target.".into(),
        ..Default::default()
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
        static_abilities: vec![],
        enters_tapped: false,
        oracle_text: "Shock deals 2 damage to any target.".into(),
        ..Default::default()
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
        static_abilities: vec![],
        enters_tapped: false,
        oracle_text: "Lava Spike deals 3 damage to target player or planeswalker.".into(),
        ..Default::default()
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
        static_abilities: vec![],
        enters_tapped: false,
        oracle_text: "Rift Bolt deals 3 damage to any target.".into(),
        ..Default::default()
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
        static_abilities: vec![],
        enters_tapped: false,
        oracle_text: "Target creature gets +3/+3 until end of turn.".into(),
        ..Default::default()
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
        static_abilities: vec![],
        enters_tapped: false,
        oracle_text: "Exile target creature. Its controller gains life equal to its power.".into(),
        ..Default::default()
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
        static_abilities: vec![],
        enters_tapped: false,
        oracle_text: "Counter target spell.".into(),
        ..Default::default()
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
        static_abilities: vec![],
        enters_tapped: false,
        oracle_text: "When Elvish Visionary enters the battlefield, draw a card.".into(),
        ..Default::default()
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
        static_abilities: vec![],
        enters_tapped: false,
        oracle_text: "When Blade Splicer enters the battlefield, create a 3/3 colorless Phyrexian Golem artifact creature token with first strike.".into(),
        ..Default::default()
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
        static_abilities: vec![],
        enters_tapped: false,
        oracle_text: "When Siege-Gang Commander enters the battlefield, create three 1/1 red Goblin creature tokens.".into(),
        ..Default::default()
    });

    // =====================================================================
    // Phase 1A Test Cards
    // =====================================================================

    // Fiery Conclusion Elemental: 2R 2/2 Elemental
    // "When ~ dies, deal 2 damage to each player."
    // Used to test the SBA recurrence loop (CR 704.3 acceptance criteria).
    db.insert(CardDef {
        id: ids::FIERY_CONCLUSION_ELEMENTAL,
        name: "Fiery Conclusion Elemental".into(),
        mana_cost: Some(ManaCost::new(2, 1, 0, 0, 0, 0)),
        card_types: vec![CardType::Creature],
        supertypes: vec![],
        subtypes: vec![Subtype("Elemental".into())],
        keywords: vec![],
        power: Some(2),
        toughness: Some(2),
        mana_abilities: vec![],
        spell_effect: None,
        activated_abilities: vec![],
        triggered_abilities: vec![TriggeredAbility {
            trigger: TriggerCondition::Dies,
            effect: Effect::DealDamage {
                amount: 2,
                target: TargetSpec::NoTarget,
            },
            description: "When Fiery Conclusion Elemental dies, it deals 2 damage to each player."
                .into(),
        }],
        starting_loyalty: None,
        static_abilities: vec![],
        enters_tapped: false,
        oracle_text: "When Fiery Conclusion Elemental dies, it deals 2 damage to each player."
            .into(),
        ..Default::default()
    });

    // Pyroclasm Elemental: 2R 3/1 Elemental
    // "When ~ dies, deal 2 damage to each creature."
    // Used to test cascading SBA — its dies trigger can kill other creatures,
    // causing recursive SBAs per CR 704.3.
    db.insert(CardDef {
        id: ids::PYROCLASM_ELEMENTAL,
        name: "Pyroclasm Elemental".into(),
        mana_cost: Some(ManaCost::new(2, 1, 0, 0, 0, 0)),
        card_types: vec![CardType::Creature],
        supertypes: vec![],
        subtypes: vec![Subtype("Elemental".into())],
        keywords: vec![],
        power: Some(3),
        toughness: Some(1),
        mana_abilities: vec![],
        spell_effect: None,
        activated_abilities: vec![],
        triggered_abilities: vec![TriggeredAbility {
            trigger: TriggerCondition::Dies,
            effect: Effect::DealDamage {
                amount: 2,
                target: TargetSpec::EachCreature,
            },
            description: "When Pyroclasm Elemental dies, it deals 2 damage to each creature."
                .into(),
        }],
        starting_loyalty: None,
        static_abilities: vec![],
        enters_tapped: false,
        oracle_text: "When Pyroclasm Elemental dies, it deals 2 damage to each creature.".into(),
        ..Default::default()
    });

    // =====================================================================
    // Phase 2A: Expanded Card Pool
    // =====================================================================

    // --- White creatures ---

    // Mother of Runes: W 1/1 Human Cleric — protection granting (simplified)
    db.insert(CardDef {
        id: ids::MOTHER_OF_RUNES,
        name: "Mother of Runes".into(),
        mana_cost: Some(ManaCost::new(0, 1, 0, 0, 0, 0)),
        card_types: vec![CardType::Creature],
        supertypes: vec![],
        subtypes: vec![Subtype("Human".into()), Subtype("Cleric".into())],
        keywords: vec![],
        power: Some(1),
        toughness: Some(1),
        mana_abilities: vec![],
        spell_effect: None,
        activated_abilities: vec![],
        triggered_abilities: vec![],
        starting_loyalty: None,
        static_abilities: vec![],
        enters_tapped: false,
        oracle_text: "{T}: Target creature you control gains protection from the color of your choice until end of turn.".into(),
        ..Default::default()
    });

    // Elite Vanguard: W 2/1 Human Soldier
    db.insert(CardDef {
        id: ids::ELITE_VANGUARD,
        name: "Elite Vanguard".into(),
        mana_cost: Some(ManaCost::new(0, 1, 0, 0, 0, 0)),
        card_types: vec![CardType::Creature],
        supertypes: vec![],
        subtypes: vec![Subtype("Human".into()), Subtype("Soldier".into())],
        keywords: vec![],
        power: Some(2),
        toughness: Some(1),
        mana_abilities: vec![],
        spell_effect: None,
        activated_abilities: vec![],
        triggered_abilities: vec![],
        starting_loyalty: None,
        static_abilities: vec![],
        enters_tapped: false,
        oracle_text: "".into(),
        ..Default::default()
    });

    // White Knight: WW 2/2 Human Knight — First strike, protection from black (simplified)
    db.insert(CardDef {
        id: ids::WHITE_KNIGHT,
        name: "White Knight".into(),
        mana_cost: Some(ManaCost::new(0, 2, 0, 0, 0, 0)),
        card_types: vec![CardType::Creature],
        supertypes: vec![],
        subtypes: vec![Subtype("Human".into()), Subtype("Knight".into())],
        keywords: vec![KeywordAbility::FirstStrike],
        power: Some(2),
        toughness: Some(2),
        mana_abilities: vec![],
        spell_effect: None,
        activated_abilities: vec![],
        triggered_abilities: vec![],
        starting_loyalty: None,
        static_abilities: vec![],
        enters_tapped: false,
        oracle_text: "First strike, protection from black".into(),
        ..Default::default()
    });

    // Leonin Skyhunter: WW 2/2 Cat Knight — Flying
    db.insert(CardDef {
        id: ids::LEONIN_SKYHUNTER,
        name: "Leonin Skyhunter".into(),
        mana_cost: Some(ManaCost::new(0, 2, 0, 0, 0, 0)),
        card_types: vec![CardType::Creature],
        supertypes: vec![],
        subtypes: vec![Subtype("Cat".into()), Subtype("Knight".into())],
        keywords: vec![KeywordAbility::Flying],
        power: Some(2),
        toughness: Some(2),
        mana_abilities: vec![],
        spell_effect: None,
        activated_abilities: vec![],
        triggered_abilities: vec![],
        starting_loyalty: None,
        static_abilities: vec![],
        enters_tapped: false,
        oracle_text: "Flying".into(),
        ..Default::default()
    });

    // Baneslayer Angel: 3WW 5/5 Angel — Flying, first strike, lifelink
    db.insert(CardDef {
        id: ids::BANESLAYER_ANGEL,
        name: "Baneslayer Angel".into(),
        mana_cost: Some(ManaCost::new(3, 2, 0, 0, 0, 0)),
        card_types: vec![CardType::Creature],
        supertypes: vec![],
        subtypes: vec![Subtype("Angel".into())],
        keywords: vec![
            KeywordAbility::Flying,
            KeywordAbility::FirstStrike,
            KeywordAbility::Lifelink,
        ],
        power: Some(5),
        toughness: Some(5),
        mana_abilities: vec![],
        spell_effect: None,
        activated_abilities: vec![],
        triggered_abilities: vec![],
        starting_loyalty: None,
        static_abilities: vec![],
        enters_tapped: false,
        oracle_text: "Flying, first strike, lifelink".into(),
        ..Default::default()
    });

    // Thalia, Guardian of Thraben: 1W 2/1 Human Soldier (simplified — no taxing)
    db.insert(CardDef {
        id: ids::THALIA_GUARDIAN,
        name: "Thalia, Guardian of Thraben".into(),
        mana_cost: Some(ManaCost::new(1, 1, 0, 0, 0, 0)),
        card_types: vec![CardType::Creature],
        supertypes: vec![Supertype::Legendary],
        subtypes: vec![Subtype("Human".into()), Subtype("Soldier".into())],
        keywords: vec![KeywordAbility::FirstStrike],
        power: Some(2),
        toughness: Some(1),
        mana_abilities: vec![],
        spell_effect: None,
        activated_abilities: vec![],
        triggered_abilities: vec![],
        starting_loyalty: None,
        static_abilities: vec![],
        enters_tapped: false,
        oracle_text: "First strike. Noncreature spells cost {1} more to cast.".into(),
        ..Default::default()
    });

    // Brimaz, King of Oreskos: 1WW 3/4 Cat Soldier — Vigilance
    db.insert(CardDef {
        id: ids::BRIMAZ_KING,
        name: "Brimaz, King of Oreskos".into(),
        mana_cost: Some(ManaCost::new(1, 2, 0, 0, 0, 0)),
        card_types: vec![CardType::Creature],
        supertypes: vec![Supertype::Legendary],
        subtypes: vec![Subtype("Cat".into()), Subtype("Soldier".into())],
        keywords: vec![KeywordAbility::Vigilance],
        power: Some(3),
        toughness: Some(4),
        mana_abilities: vec![],
        spell_effect: None,
        activated_abilities: vec![],
        triggered_abilities: vec![],
        starting_loyalty: None,
        static_abilities: vec![],
        enters_tapped: false,
        oracle_text: "Vigilance".into(),
        ..Default::default()
    });

    // Soldier of the Pantheon: W 2/1 Human Soldier
    db.insert(CardDef {
        id: ids::SOLDIER_OF_THE_PANTHEON,
        name: "Soldier of the Pantheon".into(),
        mana_cost: Some(ManaCost::new(0, 1, 0, 0, 0, 0)),
        card_types: vec![CardType::Creature],
        supertypes: vec![],
        subtypes: vec![Subtype("Human".into()), Subtype("Soldier".into())],
        keywords: vec![],
        power: Some(2),
        toughness: Some(1),
        mana_abilities: vec![],
        spell_effect: None,
        activated_abilities: vec![],
        triggered_abilities: vec![],
        starting_loyalty: None,
        static_abilities: vec![],
        enters_tapped: false,
        oracle_text: "Protection from multicolored.".into(),
        ..Default::default()
    });

    // Hero of Bladehold: 2WW 3/4 Human Knight — Battle cry (simplified as vanilla)
    db.insert(CardDef {
        id: ids::HERO_OF_BLADEHOLD,
        name: "Hero of Bladehold".into(),
        mana_cost: Some(ManaCost::new(2, 2, 0, 0, 0, 0)),
        card_types: vec![CardType::Creature],
        supertypes: vec![],
        subtypes: vec![Subtype("Human".into()), Subtype("Knight".into())],
        keywords: vec![],
        power: Some(3),
        toughness: Some(4),
        mana_abilities: vec![],
        spell_effect: None,
        activated_abilities: vec![],
        triggered_abilities: vec![],
        starting_loyalty: None,
        static_abilities: vec![],
        enters_tapped: false,
        oracle_text: "Battle cry. Whenever Hero of Bladehold attacks, create two 1/1 white Soldier creature tokens that are tapped and attacking.".into(),
        ..Default::default()
    });

    // Precinct Captain: WW 2/2 Human Soldier — First strike
    db.insert(CardDef {
        id: ids::PRECINCT_CAPTAIN,
        name: "Precinct Captain".into(),
        mana_cost: Some(ManaCost::new(0, 2, 0, 0, 0, 0)),
        card_types: vec![CardType::Creature],
        supertypes: vec![],
        subtypes: vec![Subtype("Human".into()), Subtype("Soldier".into())],
        keywords: vec![KeywordAbility::FirstStrike],
        power: Some(2),
        toughness: Some(2),
        mana_abilities: vec![],
        spell_effect: None,
        activated_abilities: vec![],
        triggered_abilities: vec![],
        starting_loyalty: None,
        static_abilities: vec![],
        enters_tapped: false,
        oracle_text: "First strike. Whenever Precinct Captain deals combat damage to a player, create a 1/1 white Soldier creature token.".into(),
        ..Default::default()
    });

    // --- White spells ---

    // Path to Exile: W — Exile target creature
    db.insert(CardDef {
        id: ids::PATH_TO_EXILE,
        name: "Path to Exile".into(),
        mana_cost: Some(ManaCost::new(0, 1, 0, 0, 0, 0)),
        card_types: vec![CardType::Instant],
        supertypes: vec![],
        subtypes: vec![],
        keywords: vec![],
        power: None,
        toughness: None,
        mana_abilities: vec![],
        spell_effect: Some(Effect::ExileTarget {
            target: TargetSpec::AnyCreature,
        }),
        activated_abilities: vec![],
        triggered_abilities: vec![],
        starting_loyalty: None,
        static_abilities: vec![],
        enters_tapped: false,
        oracle_text: "Exile target creature. Its controller may search their library for a basic land card, put that card onto the battlefield tapped, then shuffle.".into(),
        ..Default::default()
    });

    // Wrath of God: 2WW — Destroy all creatures
    db.insert(CardDef {
        id: ids::WRATH_OF_GOD,
        name: "Wrath of God".into(),
        mana_cost: Some(ManaCost::new(2, 2, 0, 0, 0, 0)),
        card_types: vec![CardType::Sorcery],
        supertypes: vec![],
        subtypes: vec![],
        keywords: vec![],
        power: None,
        toughness: None,
        mana_abilities: vec![],
        spell_effect: Some(Effect::DestroyAll),
        activated_abilities: vec![],
        triggered_abilities: vec![],
        starting_loyalty: None,
        static_abilities: vec![],
        enters_tapped: false,
        oracle_text: "Destroy all creatures. They can't be regenerated.".into(),
        ..Default::default()
    });

    // Day of Judgment: 2WW — Destroy all creatures
    db.insert(CardDef {
        id: ids::DAY_OF_JUDGMENT,
        name: "Day of Judgment".into(),
        mana_cost: Some(ManaCost::new(2, 2, 0, 0, 0, 0)),
        card_types: vec![CardType::Sorcery],
        supertypes: vec![],
        subtypes: vec![],
        keywords: vec![],
        power: None,
        toughness: None,
        mana_abilities: vec![],
        spell_effect: Some(Effect::DestroyAll),
        activated_abilities: vec![],
        triggered_abilities: vec![],
        starting_loyalty: None,
        static_abilities: vec![],
        enters_tapped: false,
        oracle_text: "Destroy all creatures.".into(),
        ..Default::default()
    });

    // Oblivion Ring: 2W — Exile target nonland permanent (simplified as destroy)
    db.insert(CardDef {
        id: ids::OBLIVION_RING,
        name: "Oblivion Ring".into(),
        mana_cost: Some(ManaCost::new(2, 1, 0, 0, 0, 0)),
        card_types: vec![CardType::Enchantment],
        supertypes: vec![],
        subtypes: vec![],
        keywords: vec![],
        power: None,
        toughness: None,
        mana_abilities: vec![],
        spell_effect: None,
        activated_abilities: vec![],
        triggered_abilities: vec![TriggeredAbility {
            trigger: TriggerCondition::EntersBattlefield,
            effect: Effect::ExileTarget {
                target: TargetSpec::AnyNonlandPermanent,
            },
            description:
                "When Oblivion Ring enters the battlefield, exile another target nonland permanent."
                    .into(),
        }],
        starting_loyalty: None,
        static_abilities: vec![],
        enters_tapped: false,
        oracle_text:
            "When Oblivion Ring enters the battlefield, exile another target nonland permanent."
                .into(),
        ..Default::default()
    });

    // Disenchant: 1W — Destroy target artifact or enchantment (simplified as destroy nonland permanent)
    db.insert(CardDef {
        id: ids::DISENCHANT,
        name: "Disenchant".into(),
        mana_cost: Some(ManaCost::new(1, 1, 0, 0, 0, 0)),
        card_types: vec![CardType::Instant],
        supertypes: vec![],
        subtypes: vec![],
        keywords: vec![],
        power: None,
        toughness: None,
        mana_abilities: vec![],
        spell_effect: Some(Effect::DestroyTarget {
            target: TargetSpec::AnyNonlandPermanent,
        }),
        activated_abilities: vec![],
        triggered_abilities: vec![],
        starting_loyalty: None,
        static_abilities: vec![],
        enters_tapped: false,
        oracle_text: "Destroy target artifact or enchantment.".into(),
        ..Default::default()
    });

    // --- White enchantments (layered effects) ---

    // Glorious Anthem: 1WW — Creatures you control get +1/+1
    db.insert(CardDef {
        id: ids::GLORIOUS_ANTHEM,
        name: "Glorious Anthem".into(),
        mana_cost: Some(ManaCost::new(1, 2, 0, 0, 0, 0)),
        card_types: vec![CardType::Enchantment],
        supertypes: vec![],
        subtypes: vec![],
        keywords: vec![],
        power: None,
        toughness: None,
        mana_abilities: vec![],
        spell_effect: None,
        activated_abilities: vec![],
        triggered_abilities: vec![],
        starting_loyalty: None,
        static_abilities: vec![StaticAbility::Anthem {
            power: 1,
            toughness: 1,
            affected: AffectedObjects::OtherCreaturesControlledBy(0), // placeholder — refreshed at runtime
        }],
        enters_tapped: false,
        oracle_text: "Creatures you control get +1/+1.".into(),
        ..Default::default()
    });

    // Honor of the Pure: 1W — White creatures you control get +1/+1 (simplified as creatures you control)
    db.insert(CardDef {
        id: ids::HONOR_OF_THE_PURE,
        name: "Honor of the Pure".into(),
        mana_cost: Some(ManaCost::new(1, 1, 0, 0, 0, 0)),
        card_types: vec![CardType::Enchantment],
        supertypes: vec![],
        subtypes: vec![],
        keywords: vec![],
        power: None,
        toughness: None,
        mana_abilities: vec![],
        spell_effect: None,
        activated_abilities: vec![],
        triggered_abilities: vec![],
        starting_loyalty: None,
        static_abilities: vec![StaticAbility::Anthem {
            power: 1,
            toughness: 1,
            affected: AffectedObjects::CreaturesControlledBy(0),
        }],
        enters_tapped: false,
        oracle_text: "White creatures you control get +1/+1.".into(),
        ..Default::default()
    });

    // Crusade: WW — White creatures get +1/+1 (simplified as all creatures)
    db.insert(CardDef {
        id: ids::CRUSADE,
        name: "Crusade".into(),
        mana_cost: Some(ManaCost::new(0, 2, 0, 0, 0, 0)),
        card_types: vec![CardType::Enchantment],
        supertypes: vec![],
        subtypes: vec![],
        keywords: vec![],
        power: None,
        toughness: None,
        mana_abilities: vec![],
        spell_effect: None,
        activated_abilities: vec![],
        triggered_abilities: vec![],
        starting_loyalty: None,
        static_abilities: vec![StaticAbility::Anthem {
            power: 1,
            toughness: 1,
            affected: AffectedObjects::AllCreatures,
        }],
        enters_tapped: false,
        oracle_text: "White creatures get +1/+1.".into(),
        ..Default::default()
    });

    // Humility: 2WW — All creatures lose all abilities and are 1/1
    db.insert(CardDef {
        id: ids::HUMILITY,
        name: "Humility".into(),
        mana_cost: Some(ManaCost::new(2, 2, 0, 0, 0, 0)),
        card_types: vec![CardType::Enchantment],
        supertypes: vec![],
        subtypes: vec![],
        keywords: vec![],
        power: None,
        toughness: None,
        mana_abilities: vec![],
        spell_effect: None,
        activated_abilities: vec![],
        triggered_abilities: vec![],
        starting_loyalty: None,
        static_abilities: vec![
            StaticAbility::RemoveAllAbilities {
                affected: AffectedObjects::AllCreatures,
            },
            StaticAbility::SetPowerToughness {
                power: 1,
                toughness: 1,
                affected: AffectedObjects::AllCreatures,
            },
        ],
        enters_tapped: false,
        oracle_text: "All creatures lose all abilities and have base power and toughness 1/1."
            .into(),
        ..Default::default()
    });

    // --- Blue creatures ---

    // Delver of Secrets: U 1/1 Human Wizard (simplified — no transform)
    db.insert(CardDef {
        id: ids::DELVER_OF_SECRETS,
        name: "Delver of Secrets".into(),
        mana_cost: Some(ManaCost::new(0, 0, 1, 0, 0, 0)),
        card_types: vec![CardType::Creature],
        supertypes: vec![],
        subtypes: vec![Subtype("Human".into()), Subtype("Wizard".into())],
        keywords: vec![],
        power: Some(1),
        toughness: Some(1),
        mana_abilities: vec![],
        spell_effect: None,
        activated_abilities: vec![],
        triggered_abilities: vec![],
        starting_loyalty: None,
        static_abilities: vec![],
        enters_tapped: false,
        oracle_text: "At the beginning of your upkeep, look at the top card of your library. You may reveal that card. If an instant or sorcery card is revealed this way, transform Delver of Secrets.".into(),
        ..Default::default()
    });

    // Snapcaster Mage: 1U 2/1 Human Wizard — Flash
    db.insert(CardDef {
        id: ids::SNAPCASTER_MAGE,
        name: "Snapcaster Mage".into(),
        mana_cost: Some(ManaCost::new(1, 0, 1, 0, 0, 0)),
        card_types: vec![CardType::Creature],
        supertypes: vec![],
        subtypes: vec![Subtype("Human".into()), Subtype("Wizard".into())],
        keywords: vec![KeywordAbility::Flash],
        power: Some(2),
        toughness: Some(1),
        mana_abilities: vec![],
        spell_effect: None,
        activated_abilities: vec![],
        triggered_abilities: vec![],
        starting_loyalty: None,
        static_abilities: vec![],
        enters_tapped: false,
        oracle_text: "Flash. When Snapcaster Mage enters the battlefield, target instant or sorcery card in your graveyard gains flashback until end of turn.".into(),
        ..Default::default()
    });

    // Vendilion Clique: 1UU 3/1 Faerie Wizard — Flash, flying
    db.insert(CardDef {
        id: ids::VENDILION_CLIQUE,
        name: "Vendilion Clique".into(),
        mana_cost: Some(ManaCost::new(1, 0, 2, 0, 0, 0)),
        card_types: vec![CardType::Creature],
        supertypes: vec![Supertype::Legendary],
        subtypes: vec![Subtype("Faerie".into()), Subtype("Wizard".into())],
        keywords: vec![KeywordAbility::Flash, KeywordAbility::Flying],
        power: Some(3),
        toughness: Some(1),
        mana_abilities: vec![],
        spell_effect: None,
        activated_abilities: vec![],
        triggered_abilities: vec![],
        starting_loyalty: None,
        static_abilities: vec![],
        enters_tapped: false,
        oracle_text: "Flash, flying. When Vendilion Clique enters the battlefield, look at target player's hand. You may choose a nonland card from it. If you do, that player reveals the chosen card, puts it on the bottom of their library, then draws a card.".into(),
        ..Default::default()
    });

    // Man-o'-War: 2U 2/2 Jellyfish — ETB bounce a creature
    db.insert(CardDef {
        id: ids::MAN_O_WAR,
        name: "Man-o'-War".into(),
        mana_cost: Some(ManaCost::new(2, 0, 1, 0, 0, 0)),
        card_types: vec![CardType::Creature],
        supertypes: vec![],
        subtypes: vec![Subtype("Jellyfish".into())],
        keywords: vec![],
        power: Some(2),
        toughness: Some(2),
        mana_abilities: vec![],
        spell_effect: None,
        activated_abilities: vec![],
        triggered_abilities: vec![TriggeredAbility {
            trigger: TriggerCondition::EntersBattlefield,
            effect: Effect::BounceTo {
                zone: ZoneType::Hand,
                target: TargetSpec::AnyCreature,
            },
            description: "When Man-o'-War enters the battlefield, return target creature to its owner's hand.".into(),
        }],
        starting_loyalty: None,
        static_abilities: vec![],
        enters_tapped: false,
        oracle_text: "When Man-o'-War enters the battlefield, return target creature to its owner's hand.".into(),
        ..Default::default()
    });

    // Serendib Efreet: 2U 3/4 Efreet — Flying
    db.insert(CardDef {
        id: ids::SERENDIB_EFREET,
        name: "Serendib Efreet".into(),
        mana_cost: Some(ManaCost::new(2, 0, 1, 0, 0, 0)),
        card_types: vec![CardType::Creature],
        supertypes: vec![],
        subtypes: vec![Subtype("Efreet".into())],
        keywords: vec![KeywordAbility::Flying],
        power: Some(3),
        toughness: Some(4),
        mana_abilities: vec![],
        spell_effect: None,
        activated_abilities: vec![],
        triggered_abilities: vec![],
        starting_loyalty: None,
        static_abilities: vec![],
        enters_tapped: false,
        oracle_text: "Flying. At the beginning of your upkeep, Serendib Efreet deals 1 damage to you.".into(),
        ..Default::default()
    });

    // Phantasmal Bear: U 2/2 Bear Illusion
    db.insert(CardDef {
        id: ids::PHANTASMAL_BEAR,
        name: "Phantasmal Bear".into(),
        mana_cost: Some(ManaCost::new(0, 0, 1, 0, 0, 0)),
        card_types: vec![CardType::Creature],
        supertypes: vec![],
        subtypes: vec![Subtype("Bear".into()), Subtype("Illusion".into())],
        keywords: vec![],
        power: Some(2),
        toughness: Some(2),
        mana_abilities: vec![],
        spell_effect: None,
        activated_abilities: vec![],
        triggered_abilities: vec![],
        starting_loyalty: None,
        static_abilities: vec![],
        enters_tapped: false,
        oracle_text: "When Phantasmal Bear becomes the target of a spell or ability, sacrifice it."
            .into(),
        ..Default::default()
    });

    // --- Blue spells ---

    // Mana Leak: 1U — Counter target spell unless its controller pays {3}
    db.insert(CardDef {
        id: ids::MANA_LEAK,
        name: "Mana Leak".into(),
        mana_cost: Some(ManaCost::new(1, 0, 1, 0, 0, 0)),
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
        static_abilities: vec![],
        enters_tapped: false,
        oracle_text: "Counter target spell unless its controller pays {3}.".into(),
        ..Default::default()
    });

    // Remand: 1U — Counter target spell, draw a card (simplified as counter + draw)
    db.insert(CardDef {
        id: ids::REMAND,
        name: "Remand".into(),
        mana_cost: Some(ManaCost::new(1, 0, 1, 0, 0, 0)),
        card_types: vec![CardType::Instant],
        supertypes: vec![],
        subtypes: vec![],
        keywords: vec![],
        power: None,
        toughness: None,
        mana_abilities: vec![],
        spell_effect: Some(Effect::Multiple(vec![
            Effect::Counter { target: TargetSpec::AnySpell },
            Effect::DrawCards { count: 1 },
        ])),
        activated_abilities: vec![],
        triggered_abilities: vec![],
        starting_loyalty: None,
        static_abilities: vec![],
        enters_tapped: false,
        oracle_text: "Counter target spell. If that spell is countered this way, put it into its owner's hand instead of into that player's graveyard. Draw a card.".into(),
        ..Default::default()
    });

    // Brainstorm: U — Draw 3, then put 2 cards from hand on top (partially simplified: draw 3)
    db.insert(CardDef {
        id: ids::BRAINSTORM,
        name: "Brainstorm".into(),
        mana_cost: Some(ManaCost::new(0, 0, 1, 0, 0, 0)),
        card_types: vec![CardType::Instant],
        supertypes: vec![],
        subtypes: vec![],
        keywords: vec![],
        power: None,
        toughness: None,
        mana_abilities: vec![],
        spell_effect: Some(Effect::DrawCards { count: 3 }),
        activated_abilities: vec![],
        triggered_abilities: vec![],
        starting_loyalty: None,
        static_abilities: vec![],
        enters_tapped: false,
        oracle_text: "Draw three cards, then put two cards from your hand on top of your library in any order.".into(),
        ..Default::default()
    });

    // Ponder: U — Look at top 3, may shuffle, draw (simplified as draw 1)
    db.insert(CardDef {
        id: ids::PONDER,
        name: "Ponder".into(),
        mana_cost: Some(ManaCost::new(0, 0, 1, 0, 0, 0)),
        card_types: vec![CardType::Sorcery],
        supertypes: vec![],
        subtypes: vec![],
        keywords: vec![],
        power: None,
        toughness: None,
        mana_abilities: vec![],
        spell_effect: Some(Effect::DrawCards { count: 1 }),
        activated_abilities: vec![],
        triggered_abilities: vec![],
        starting_loyalty: None,
        static_abilities: vec![],
        enters_tapped: false,
        oracle_text: "Look at the top three cards of your library, then put them back in any order. You may shuffle. Draw a card.".into(),
        ..Default::default()
    });

    // Unsummon: U — Return target creature to its owner's hand
    db.insert(CardDef {
        id: ids::UNSUMMON,
        name: "Unsummon".into(),
        mana_cost: Some(ManaCost::new(0, 0, 1, 0, 0, 0)),
        card_types: vec![CardType::Instant],
        supertypes: vec![],
        subtypes: vec![],
        keywords: vec![],
        power: None,
        toughness: None,
        mana_abilities: vec![],
        spell_effect: Some(Effect::BounceTo {
            zone: ZoneType::Hand,
            target: TargetSpec::AnyCreature,
        }),
        activated_abilities: vec![],
        triggered_abilities: vec![],
        starting_loyalty: None,
        static_abilities: vec![],
        enters_tapped: false,
        oracle_text: "Return target creature to its owner's hand.".into(),
        ..Default::default()
    });

    // --- Black creatures ---

    // Dark Confidant: 1B 2/1 Human Wizard
    db.insert(CardDef {
        id: ids::DARK_CONFIDANT,
        name: "Dark Confidant".into(),
        mana_cost: Some(ManaCost::new(1, 0, 0, 1, 0, 0)),
        card_types: vec![CardType::Creature],
        supertypes: vec![],
        subtypes: vec![Subtype("Human".into()), Subtype("Wizard".into())],
        keywords: vec![],
        power: Some(2),
        toughness: Some(1),
        mana_abilities: vec![],
        spell_effect: None,
        activated_abilities: vec![],
        triggered_abilities: vec![],
        starting_loyalty: None,
        static_abilities: vec![],
        enters_tapped: false,
        oracle_text: "At the beginning of your upkeep, reveal the top card of your library and put that card into your hand. You lose life equal to its mana value.".into(),
        ..Default::default()
    });

    // Hypnotic Specter: 1BB 2/2 Specter — Flying
    db.insert(CardDef {
        id: ids::HYPNOTIC_SPECTER,
        name: "Hypnotic Specter".into(),
        mana_cost: Some(ManaCost::new(1, 0, 0, 2, 0, 0)),
        card_types: vec![CardType::Creature],
        supertypes: vec![],
        subtypes: vec![Subtype("Specter".into())],
        keywords: vec![KeywordAbility::Flying],
        power: Some(2),
        toughness: Some(2),
        mana_abilities: vec![],
        spell_effect: None,
        activated_abilities: vec![],
        triggered_abilities: vec![],
        starting_loyalty: None,
        static_abilities: vec![],
        enters_tapped: false,
        oracle_text: "Flying. Whenever Hypnotic Specter deals damage to an opponent, that player discards a card at random.".into(),
        ..Default::default()
    });

    // Nantuko Shade: BB 2/1 Insect Shade
    db.insert(CardDef {
        id: ids::NANTUKO_SHADE,
        name: "Nantuko Shade".into(),
        mana_cost: Some(ManaCost::new(0, 0, 0, 2, 0, 0)),
        card_types: vec![CardType::Creature],
        supertypes: vec![],
        subtypes: vec![Subtype("Insect".into()), Subtype("Shade".into())],
        keywords: vec![],
        power: Some(2),
        toughness: Some(1),
        mana_abilities: vec![],
        spell_effect: None,
        activated_abilities: vec![],
        triggered_abilities: vec![],
        starting_loyalty: None,
        static_abilities: vec![],
        enters_tapped: false,
        oracle_text: "{B}: Nantuko Shade gets +1/+1 until end of turn.".into(),
        ..Default::default()
    });

    // Vampire Nighthawk: 1BB 2/3 Vampire Shaman — Flying, deathtouch, lifelink
    db.insert(CardDef {
        id: ids::VAMPIRE_NIGHTHAWK,
        name: "Vampire Nighthawk".into(),
        mana_cost: Some(ManaCost::new(1, 0, 0, 2, 0, 0)),
        card_types: vec![CardType::Creature],
        supertypes: vec![],
        subtypes: vec![Subtype("Vampire".into()), Subtype("Shaman".into())],
        keywords: vec![
            KeywordAbility::Flying,
            KeywordAbility::Deathtouch,
            KeywordAbility::Lifelink,
        ],
        power: Some(2),
        toughness: Some(3),
        mana_abilities: vec![],
        spell_effect: None,
        activated_abilities: vec![],
        triggered_abilities: vec![],
        starting_loyalty: None,
        static_abilities: vec![],
        enters_tapped: false,
        oracle_text: "Flying, deathtouch, lifelink".into(),
        ..Default::default()
    });

    // Gatekeeper of Malakir: BB 2/2 Vampire Warrior — Kicker B, ETB sacrifice if kicked (simplified)
    db.insert(CardDef {
        id: ids::GATEKEEPER_OF_MALAKIR,
        name: "Gatekeeper of Malakir".into(),
        mana_cost: Some(ManaCost::new(0, 0, 0, 2, 0, 0)),
        card_types: vec![CardType::Creature],
        supertypes: vec![],
        subtypes: vec![Subtype("Vampire".into()), Subtype("Warrior".into())],
        keywords: vec![],
        power: Some(2),
        toughness: Some(2),
        mana_abilities: vec![],
        spell_effect: None,
        activated_abilities: vec![],
        triggered_abilities: vec![],
        starting_loyalty: None,
        static_abilities: vec![],
        enters_tapped: false,
        oracle_text: "Kicker {B}. When Gatekeeper of Malakir enters the battlefield, if it was kicked, target player sacrifices a creature.".into(),
        ..Default::default()
    });

    // Bloodghast: BB 2/1 Vampire Spirit — can't block (simplified)
    db.insert(CardDef {
        id: ids::BLOODGHAST,
        name: "Bloodghast".into(),
        mana_cost: Some(ManaCost::new(0, 0, 0, 2, 0, 0)),
        card_types: vec![CardType::Creature],
        supertypes: vec![],
        subtypes: vec![Subtype("Vampire".into()), Subtype("Spirit".into())],
        keywords: vec![],
        power: Some(2),
        toughness: Some(1),
        mana_abilities: vec![],
        spell_effect: None,
        activated_abilities: vec![],
        triggered_abilities: vec![],
        starting_loyalty: None,
        static_abilities: vec![],
        enters_tapped: false,
        oracle_text: "Bloodghast can't block. Landfall — Whenever a land enters the battlefield under your control, you may return Bloodghast from your graveyard to the battlefield.".into(),
        ..Default::default()
    });

    // Geralf's Messenger: BBB 3/2 Zombie — ETB opponent loses 2 life
    db.insert(CardDef {
        id: ids::GERALF_MESSENGER,
        name: "Geralf's Messenger".into(),
        mana_cost: Some(ManaCost::new(0, 0, 0, 3, 0, 0)),
        card_types: vec![CardType::Creature],
        supertypes: vec![],
        subtypes: vec![Subtype("Zombie".into())],
        keywords: vec![],
        power: Some(3),
        toughness: Some(2),
        mana_abilities: vec![],
        spell_effect: None,
        activated_abilities: vec![],
        triggered_abilities: vec![TriggeredAbility {
            trigger: TriggerCondition::EntersBattlefield,
            effect: Effect::LoseLife {
                amount: 2,
                target: TargetSpec::Opponent,
            },
            description: "When Geralf's Messenger enters the battlefield, target opponent loses 2 life.".into(),
        }],
        starting_loyalty: None,
        static_abilities: vec![],
        enters_tapped: true,
        oracle_text: "Geralf's Messenger enters the battlefield tapped. When Geralf's Messenger enters the battlefield, target opponent loses 2 life. Undying.".into(),
        ..Default::default()
    });

    // Phyrexian Obliterator: BBBB 5/5 Phyrexian Horror — Trample
    db.insert(CardDef {
        id: ids::PHYREXIAN_OBLITERATOR,
        name: "Phyrexian Obliterator".into(),
        mana_cost: Some(ManaCost::new(0, 0, 0, 4, 0, 0)),
        card_types: vec![CardType::Creature],
        supertypes: vec![],
        subtypes: vec![Subtype("Phyrexian".into()), Subtype("Horror".into())],
        keywords: vec![KeywordAbility::Trample],
        power: Some(5),
        toughness: Some(5),
        mana_abilities: vec![],
        spell_effect: None,
        activated_abilities: vec![],
        triggered_abilities: vec![],
        starting_loyalty: None,
        static_abilities: vec![],
        enters_tapped: false,
        oracle_text: "Trample. Whenever a source deals damage to Phyrexian Obliterator, that source's controller sacrifices that many permanents.".into(),
        ..Default::default()
    });

    // Knight of the Ebon Legion: B 1/2 Vampire Knight
    db.insert(CardDef {
        id: ids::KNIGHT_OF_THE_EBON_LEGION,
        name: "Knight of the Ebon Legion".into(),
        mana_cost: Some(ManaCost::new(0, 0, 0, 1, 0, 0)),
        card_types: vec![CardType::Creature],
        supertypes: vec![],
        subtypes: vec![Subtype("Vampire".into()), Subtype("Knight".into())],
        keywords: vec![],
        power: Some(1),
        toughness: Some(2),
        mana_abilities: vec![],
        spell_effect: None,
        activated_abilities: vec![],
        triggered_abilities: vec![],
        starting_loyalty: None,
        static_abilities: vec![],
        enters_tapped: false,
        oracle_text:
            "{2}{B}: Knight of the Ebon Legion gets +3/+3 and gains deathtouch until end of turn."
                .into(),
        ..Default::default()
    });

    // --- Black spells ---

    // Doom Blade: 1B — Destroy target nonblack creature (simplified as destroy creature)
    db.insert(CardDef {
        id: ids::DOOM_BLADE,
        name: "Doom Blade".into(),
        mana_cost: Some(ManaCost::new(1, 0, 0, 1, 0, 0)),
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
        activated_abilities: vec![],
        triggered_abilities: vec![],
        starting_loyalty: None,
        static_abilities: vec![],
        enters_tapped: false,
        oracle_text: "Destroy target nonblack creature.".into(),
        ..Default::default()
    });

    // Go for the Throat: 1B — Destroy target nonartifact creature (simplified as destroy creature)
    db.insert(CardDef {
        id: ids::GO_FOR_THE_THROAT,
        name: "Go for the Throat".into(),
        mana_cost: Some(ManaCost::new(1, 0, 0, 1, 0, 0)),
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
        activated_abilities: vec![],
        triggered_abilities: vec![],
        starting_loyalty: None,
        static_abilities: vec![],
        enters_tapped: false,
        oracle_text: "Destroy target nonartifact creature.".into(),
        ..Default::default()
    });

    // Thoughtseize: B — Target player discards a card (simplified)
    db.insert(CardDef {
        id: ids::THOUGHTSEIZE,
        name: "Thoughtseize".into(),
        mana_cost: Some(ManaCost::new(0, 0, 0, 1, 0, 0)),
        card_types: vec![CardType::Sorcery],
        supertypes: vec![],
        subtypes: vec![],
        keywords: vec![],
        power: None,
        toughness: None,
        mana_abilities: vec![],
        spell_effect: Some(Effect::Multiple(vec![
            Effect::DiscardCards { count: 1, target: TargetSpec::Opponent },
            Effect::LoseLife { amount: 2, target: TargetSpec::Controller },
        ])),
        activated_abilities: vec![],
        triggered_abilities: vec![],
        starting_loyalty: None,
        static_abilities: vec![],
        enters_tapped: false,
        oracle_text: "Target player reveals their hand. You choose a nonland card from it. That player discards that card. You lose 2 life.".into(),
        ..Default::default()
    });

    // Hymn to Tourach: BB — Target player discards 2 cards at random
    db.insert(CardDef {
        id: ids::HYMN_TO_TOURACH,
        name: "Hymn to Tourach".into(),
        mana_cost: Some(ManaCost::new(0, 0, 0, 2, 0, 0)),
        card_types: vec![CardType::Sorcery],
        supertypes: vec![],
        subtypes: vec![],
        keywords: vec![],
        power: None,
        toughness: None,
        mana_abilities: vec![],
        spell_effect: Some(Effect::DiscardCards {
            count: 2,
            target: TargetSpec::Opponent,
        }),
        activated_abilities: vec![],
        triggered_abilities: vec![],
        starting_loyalty: None,
        static_abilities: vec![],
        enters_tapped: false,
        oracle_text: "Target player discards two cards at random.".into(),
        ..Default::default()
    });

    // Diabolic Edict: 1B — Target player sacrifices a creature
    db.insert(CardDef {
        id: ids::DIABOLIC_EDICT,
        name: "Diabolic Edict".into(),
        mana_cost: Some(ManaCost::new(1, 0, 0, 1, 0, 0)),
        card_types: vec![CardType::Instant],
        supertypes: vec![],
        subtypes: vec![],
        keywords: vec![],
        power: None,
        toughness: None,
        mana_abilities: vec![],
        spell_effect: Some(Effect::SacrificeCreatures {
            count: 1,
            target: TargetSpec::Opponent,
        }),
        activated_abilities: vec![],
        triggered_abilities: vec![],
        starting_loyalty: None,
        static_abilities: vec![],
        enters_tapped: false,
        oracle_text: "Target player sacrifices a creature.".into(),
        ..Default::default()
    });

    // Tragic Slip: B — Target creature gets -1/-1 until end of turn (simplified)
    db.insert(CardDef {
        id: ids::TRAGIC_SLIP,
        name: "Tragic Slip".into(),
        mana_cost: Some(ManaCost::new(0, 0, 0, 1, 0, 0)),
        card_types: vec![CardType::Instant],
        supertypes: vec![],
        subtypes: vec![],
        keywords: vec![],
        power: None,
        toughness: None,
        mana_abilities: vec![],
        spell_effect: Some(Effect::Debuff {
            power: -1,
            toughness: -1,
            until_eot: true,
        }),
        activated_abilities: vec![],
        triggered_abilities: vec![],
        starting_loyalty: None,
        static_abilities: vec![],
        enters_tapped: false,
        oracle_text: "Target creature gets -1/-1 until end of turn. Morbid — That creature gets -13/-13 until end of turn instead if a creature died this turn.".into(),
        ..Default::default()
    });

    // --- Red creatures ---

    // Ash Zealot: RR 2/2 Human Warrior — First strike, haste
    db.insert(CardDef {
        id: ids::ASH_ZEALOT,
        name: "Ash Zealot".into(),
        mana_cost: Some(ManaCost::new(0, 0, 0, 0, 2, 0)),
        card_types: vec![CardType::Creature],
        supertypes: vec![],
        subtypes: vec![Subtype("Human".into()), Subtype("Warrior".into())],
        keywords: vec![KeywordAbility::FirstStrike, KeywordAbility::Haste],
        power: Some(2),
        toughness: Some(2),
        mana_abilities: vec![],
        spell_effect: None,
        activated_abilities: vec![],
        triggered_abilities: vec![],
        starting_loyalty: None,
        static_abilities: vec![],
        enters_tapped: false,
        oracle_text: "First strike, haste".into(),
        ..Default::default()
    });

    // Ember Hauler: RR 2/2 Goblin — sac: deal 2 damage (simplified as vanilla)
    db.insert(CardDef {
        id: ids::EMBER_HAULER,
        name: "Ember Hauler".into(),
        mana_cost: Some(ManaCost::new(0, 0, 0, 0, 2, 0)),
        card_types: vec![CardType::Creature],
        supertypes: vec![],
        subtypes: vec![Subtype("Goblin".into())],
        keywords: vec![],
        power: Some(2),
        toughness: Some(2),
        mana_abilities: vec![],
        spell_effect: None,
        activated_abilities: vec![],
        triggered_abilities: vec![],
        starting_loyalty: None,
        static_abilities: vec![],
        enters_tapped: false,
        oracle_text: "{1}, Sacrifice Ember Hauler: Ember Hauler deals 2 damage to any target."
            .into(),
        ..Default::default()
    });

    // Hellrider: 2RR 3/3 Devil — Haste
    db.insert(CardDef {
        id: ids::HELLRIDER,
        name: "Hellrider".into(),
        mana_cost: Some(ManaCost::new(2, 0, 0, 0, 2, 0)),
        card_types: vec![CardType::Creature],
        supertypes: vec![],
        subtypes: vec![Subtype("Devil".into())],
        keywords: vec![KeywordAbility::Haste],
        power: Some(3),
        toughness: Some(3),
        mana_abilities: vec![],
        spell_effect: None,
        activated_abilities: vec![],
        triggered_abilities: vec![],
        starting_loyalty: None,
        static_abilities: vec![],
        enters_tapped: false,
        oracle_text: "Haste. Whenever a creature you control attacks, Hellrider deals 1 damage to the defending player.".into(),
        ..Default::default()
    });

    // Jackal Pup: R 2/1 Hound
    db.insert(CardDef {
        id: ids::JACKAL_PUP,
        name: "Jackal Pup".into(),
        mana_cost: Some(ManaCost::new(0, 0, 0, 0, 1, 0)),
        card_types: vec![CardType::Creature],
        supertypes: vec![],
        subtypes: vec![Subtype("Hound".into())],
        keywords: vec![],
        power: Some(2),
        toughness: Some(1),
        mana_abilities: vec![],
        spell_effect: None,
        activated_abilities: vec![],
        triggered_abilities: vec![],
        starting_loyalty: None,
        static_abilities: vec![],
        enters_tapped: false,
        oracle_text: "Whenever Jackal Pup is dealt damage, it deals that much damage to you."
            .into(),
        ..Default::default()
    });

    // Keldon Marauders: 1R 3/3 Human Warrior — Vanishing 2 (simplified)
    db.insert(CardDef {
        id: ids::KELDON_MARAUDERS,
        name: "Keldon Marauders".into(),
        mana_cost: Some(ManaCost::new(1, 0, 0, 0, 1, 0)),
        card_types: vec![CardType::Creature],
        supertypes: vec![],
        subtypes: vec![Subtype("Human".into()), Subtype("Warrior".into())],
        keywords: vec![],
        power: Some(3),
        toughness: Some(3),
        mana_abilities: vec![],
        spell_effect: None,
        activated_abilities: vec![],
        triggered_abilities: vec![TriggeredAbility {
            trigger: TriggerCondition::EntersBattlefield,
            effect: Effect::DealDamage {
                amount: 1,
                target: TargetSpec::Opponent,
            },
            description: "When Keldon Marauders enters the battlefield, deal 1 damage to target opponent.".into(),
        }],
        starting_loyalty: None,
        static_abilities: vec![],
        enters_tapped: false,
        oracle_text: "Vanishing 2. When Keldon Marauders enters or leaves the battlefield, it deals 1 damage to target player or planeswalker.".into(),
        ..Default::default()
    });

    // Vexing Devil: R 4/3 Devil
    db.insert(CardDef {
        id: ids::VEXING_DEVIL,
        name: "Vexing Devil".into(),
        mana_cost: Some(ManaCost::new(0, 0, 0, 0, 1, 0)),
        card_types: vec![CardType::Creature],
        supertypes: vec![],
        subtypes: vec![Subtype("Devil".into())],
        keywords: vec![],
        power: Some(4),
        toughness: Some(3),
        mana_abilities: vec![],
        spell_effect: None,
        activated_abilities: vec![],
        triggered_abilities: vec![],
        starting_loyalty: None,
        static_abilities: vec![],
        enters_tapped: false,
        oracle_text: "When Vexing Devil enters the battlefield, any opponent may have it deal 4 damage to them. If a player does, sacrifice Vexing Devil.".into(),
        ..Default::default()
    });

    // Eidolon of the Great Revel: RR 2/2 Spirit
    db.insert(CardDef {
        id: ids::EIDOLON_OF_GREAT_REVEL,
        name: "Eidolon of the Great Revel".into(),
        mana_cost: Some(ManaCost::new(0, 0, 0, 0, 2, 0)),
        card_types: vec![CardType::Creature],
        supertypes: vec![],
        subtypes: vec![Subtype("Spirit".into())],
        keywords: vec![],
        power: Some(2),
        toughness: Some(2),
        mana_abilities: vec![],
        spell_effect: None,
        activated_abilities: vec![],
        triggered_abilities: vec![],
        starting_loyalty: None,
        static_abilities: vec![],
        enters_tapped: false,
        oracle_text: "Whenever a player casts a spell with mana value 3 or less, Eidolon of the Great Revel deals 2 damage to that player.".into(),
        ..Default::default()
    });

    // Young Pyromancer: 1R 2/1 Human Shaman
    db.insert(CardDef {
        id: ids::YOUNG_PYROMANCER,
        name: "Young Pyromancer".into(),
        mana_cost: Some(ManaCost::new(1, 0, 0, 0, 1, 0)),
        card_types: vec![CardType::Creature],
        supertypes: vec![],
        subtypes: vec![Subtype("Human".into()), Subtype("Shaman".into())],
        keywords: vec![],
        power: Some(2),
        toughness: Some(1),
        mana_abilities: vec![],
        spell_effect: None,
        activated_abilities: vec![],
        triggered_abilities: vec![],
        starting_loyalty: None,
        static_abilities: vec![],
        enters_tapped: false,
        oracle_text: "Whenever you cast an instant or sorcery spell, create a 1/1 red Elemental creature token.".into(),
        ..Default::default()
    });

    // Goblin Chainwhirler: RRR 3/3 Goblin Warrior — First strike, ETB 1 damage to each opponent and each creature/planeswalker they control
    db.insert(CardDef {
        id: ids::GOBLIN_CHAINWHIRLER,
        name: "Goblin Chainwhirler".into(),
        mana_cost: Some(ManaCost::new(0, 0, 0, 0, 3, 0)),
        card_types: vec![CardType::Creature],
        supertypes: vec![],
        subtypes: vec![Subtype("Goblin".into()), Subtype("Warrior".into())],
        keywords: vec![KeywordAbility::FirstStrike],
        power: Some(3),
        toughness: Some(3),
        mana_abilities: vec![],
        spell_effect: None,
        activated_abilities: vec![],
        triggered_abilities: vec![TriggeredAbility {
            trigger: TriggerCondition::EntersBattlefield,
            effect: Effect::DealDamage {
                amount: 1,
                target: TargetSpec::EachCreature,
            },
            description: "When Goblin Chainwhirler enters the battlefield, it deals 1 damage to each opponent and each creature and planeswalker they control.".into(),
        }],
        starting_loyalty: None,
        static_abilities: vec![],
        enters_tapped: false,
        oracle_text: "First strike. When Goblin Chainwhirler enters the battlefield, it deals 1 damage to each opponent and each creature and planeswalker they control.".into(),
        ..Default::default()
    });

    // --- Red spells ---

    // Chain Lightning: R — Deal 3 damage to any target
    db.insert(CardDef {
        id: ids::CHAIN_LIGHTNING,
        name: "Chain Lightning".into(),
        mana_cost: Some(ManaCost::new(0, 0, 0, 0, 1, 0)),
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
        static_abilities: vec![],
        enters_tapped: false,
        oracle_text: "Chain Lightning deals 3 damage to any target.".into(),
        ..Default::default()
    });

    // Searing Blaze: RR — Deal 1 damage to target player and 1 to target creature (simplified as 3 to creature)
    db.insert(CardDef {
        id: ids::SEARING_BLAZE,
        name: "Searing Blaze".into(),
        mana_cost: Some(ManaCost::new(0, 0, 0, 0, 2, 0)),
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
        static_abilities: vec![],
        enters_tapped: false,
        oracle_text: "Searing Blaze deals 1 damage to target player or planeswalker and 1 damage to target creature that player or that planeswalker's controller controls. Landfall — If you had a land enter the battlefield under your control this turn, Searing Blaze deals 3 damage to that player or planeswalker and 3 damage to that creature instead.".into(),
        ..Default::default()
    });

    // Skullcrack: 1R — Deal 3 damage to opponent, can't gain life
    db.insert(CardDef {
        id: ids::SKULLCRACK,
        name: "Skullcrack".into(),
        mana_cost: Some(ManaCost::new(1, 0, 0, 0, 1, 0)),
        card_types: vec![CardType::Instant],
        supertypes: vec![],
        subtypes: vec![],
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
        static_abilities: vec![],
        enters_tapped: false,
        oracle_text: "Players can't gain life this turn. Damage can't be prevented this turn. Skullcrack deals 3 damage to target player or planeswalker.".into(),
        ..Default::default()
    });

    // Flames of the Blood Hand: 2R — Deal 4 to any player
    db.insert(CardDef {
        id: ids::FLAMES_OF_THE_BLOOD_HAND,
        name: "Flames of the Blood Hand".into(),
        mana_cost: Some(ManaCost::new(2, 0, 0, 0, 1, 0)),
        card_types: vec![CardType::Instant],
        supertypes: vec![],
        subtypes: vec![],
        keywords: vec![],
        power: None,
        toughness: None,
        mana_abilities: vec![],
        spell_effect: Some(Effect::DealDamage {
            amount: 4,
            target: TargetSpec::AnyPlayer,
        }),
        activated_abilities: vec![],
        triggered_abilities: vec![],
        starting_loyalty: None,
        static_abilities: vec![],
        enters_tapped: false,
        oracle_text: "Flames of the Blood Hand deals 4 damage to target player or planeswalker. The damage can't be prevented, and if a player would gain life this turn, that player gains no life instead.".into(),
        ..Default::default()
    });

    // Searing Blood: RR — Deal 2 damage to target creature, 3 to controller if it dies
    db.insert(CardDef {
        id: ids::SEARING_BLOOD,
        name: "Searing Blood".into(),
        mana_cost: Some(ManaCost::new(0, 0, 0, 0, 2, 0)),
        card_types: vec![CardType::Instant],
        supertypes: vec![],
        subtypes: vec![],
        keywords: vec![],
        power: None,
        toughness: None,
        mana_abilities: vec![],
        spell_effect: Some(Effect::DealDamage {
            amount: 2,
            target: TargetSpec::AnyCreature,
        }),
        activated_abilities: vec![],
        triggered_abilities: vec![],
        starting_loyalty: None,
        static_abilities: vec![],
        enters_tapped: false,
        oracle_text: "Searing Blood deals 2 damage to target creature. When that creature dies this turn, Searing Blood deals 3 damage to the creature's controller.".into(),
        ..Default::default()
    });

    // --- Green creatures ---

    // Tarmogoyf: 1G 0/1 Lhurgoyf (simplified — no graveyard counting CDA)
    db.insert(CardDef {
        id: ids::TARMOGOYF,
        name: "Tarmogoyf".into(),
        mana_cost: Some(ManaCost::new(1, 0, 0, 0, 0, 1)),
        card_types: vec![CardType::Creature],
        supertypes: vec![],
        subtypes: vec![Subtype("Lhurgoyf".into())],
        keywords: vec![],
        power: Some(0),
        toughness: Some(1),
        mana_abilities: vec![],
        spell_effect: None,
        activated_abilities: vec![],
        triggered_abilities: vec![],
        starting_loyalty: None,
        static_abilities: vec![],
        enters_tapped: false,
        oracle_text: "Tarmogoyf's power is equal to the number of card types among cards in all graveyards and its toughness is equal to that number plus 1.".into(),
        ..Default::default()
    });

    // Scavenging Ooze: 1G 2/2 Ooze
    db.insert(CardDef {
        id: ids::SCAVENGING_OOZE,
        name: "Scavenging Ooze".into(),
        mana_cost: Some(ManaCost::new(1, 0, 0, 0, 0, 1)),
        card_types: vec![CardType::Creature],
        supertypes: vec![],
        subtypes: vec![Subtype("Ooze".into())],
        keywords: vec![],
        power: Some(2),
        toughness: Some(2),
        mana_abilities: vec![],
        spell_effect: None,
        activated_abilities: vec![],
        triggered_abilities: vec![],
        starting_loyalty: None,
        static_abilities: vec![],
        enters_tapped: false,
        oracle_text: "{G}: Exile target card from a graveyard. If it was a creature card, put a +1/+1 counter on Scavenging Ooze and you gain 1 life.".into(),
        ..Default::default()
    });

    // Strangleroot Geist: GG 2/1 Spirit — Haste, undying (simplified)
    db.insert(CardDef {
        id: ids::STRANGLEROOT_GEIST,
        name: "Strangleroot Geist".into(),
        mana_cost: Some(ManaCost::new(0, 0, 0, 0, 0, 2)),
        card_types: vec![CardType::Creature],
        supertypes: vec![],
        subtypes: vec![Subtype("Spirit".into())],
        keywords: vec![KeywordAbility::Haste],
        power: Some(2),
        toughness: Some(1),
        mana_abilities: vec![],
        spell_effect: None,
        activated_abilities: vec![],
        triggered_abilities: vec![],
        starting_loyalty: None,
        static_abilities: vec![],
        enters_tapped: false,
        oracle_text: "Haste. Undying.".into(),
        ..Default::default()
    });

    // Wild Nacatl: G 1/1 Cat Warrior (simplified — no domain bonus)
    db.insert(CardDef {
        id: ids::WILD_NACATL,
        name: "Wild Nacatl".into(),
        mana_cost: Some(ManaCost::new(0, 0, 0, 0, 0, 1)),
        card_types: vec![CardType::Creature],
        supertypes: vec![],
        subtypes: vec![Subtype("Cat".into()), Subtype("Warrior".into())],
        keywords: vec![],
        power: Some(1),
        toughness: Some(1),
        mana_abilities: vec![],
        spell_effect: None,
        activated_abilities: vec![],
        triggered_abilities: vec![],
        starting_loyalty: None,
        static_abilities: vec![],
        enters_tapped: false,
        oracle_text: "Wild Nacatl gets +1/+1 as long as you control a Mountain. Wild Nacatl gets +1/+1 as long as you control a Plains.".into(),
        ..Default::default()
    });

    // Experiment One: G 1/1 Human Ooze
    db.insert(CardDef {
        id: ids::EXPERIMENT_ONE,
        name: "Experiment One".into(),
        mana_cost: Some(ManaCost::new(0, 0, 0, 0, 0, 1)),
        card_types: vec![CardType::Creature],
        supertypes: vec![],
        subtypes: vec![Subtype("Human".into()), Subtype("Ooze".into())],
        keywords: vec![],
        power: Some(1),
        toughness: Some(1),
        mana_abilities: vec![],
        spell_effect: None,
        activated_abilities: vec![],
        triggered_abilities: vec![],
        starting_loyalty: None,
        static_abilities: vec![],
        enters_tapped: false,
        oracle_text:
            "Evolve. Remove two +1/+1 counters from Experiment One: Regenerate Experiment One."
                .into(),
        ..Default::default()
    });

    // Dryad Militant: G/W 2/1 Dryad Soldier
    db.insert(CardDef {
        id: ids::DRYAD_MILITANT,
        name: "Dryad Militant".into(),
        mana_cost: Some(ManaCost::new(0, 0, 0, 0, 0, 1)),
        card_types: vec![CardType::Creature],
        supertypes: vec![],
        subtypes: vec![Subtype("Dryad".into()), Subtype("Soldier".into())],
        keywords: vec![],
        power: Some(2),
        toughness: Some(1),
        mana_abilities: vec![],
        spell_effect: None,
        activated_abilities: vec![],
        triggered_abilities: vec![],
        starting_loyalty: None,
        static_abilities: vec![],
        enters_tapped: false,
        oracle_text: "If an instant or sorcery card would be put into a graveyard from anywhere, exile it instead.".into(),
        ..Default::default()
    });

    // Thrun, the Last Troll: 2GG 4/4 Troll Shaman — trample (simplified, no hexproof/regen)
    db.insert(CardDef {
        id: ids::THRUN_LAST_TROLL,
        name: "Thrun, the Last Troll".into(),
        mana_cost: Some(ManaCost::new(2, 0, 0, 0, 0, 2)),
        card_types: vec![CardType::Creature],
        supertypes: vec![Supertype::Legendary],
        subtypes: vec![Subtype("Troll".into()), Subtype("Shaman".into())],
        keywords: vec![KeywordAbility::Trample],
        power: Some(4),
        toughness: Some(4),
        mana_abilities: vec![],
        spell_effect: None,
        activated_abilities: vec![],
        triggered_abilities: vec![],
        starting_loyalty: None,
        static_abilities: vec![],
        enters_tapped: false,
        oracle_text:
            "This spell can't be countered. Hexproof. {1}{G}: Regenerate Thrun, the Last Troll."
                .into(),
        ..Default::default()
    });

    // Rancor Beast: 2G 3/2 Beast — Trample
    db.insert(CardDef {
        id: ids::RANCOR_BEAST,
        name: "Rancor Beast".into(),
        mana_cost: Some(ManaCost::new(2, 0, 0, 0, 0, 1)),
        card_types: vec![CardType::Creature],
        supertypes: vec![],
        subtypes: vec![Subtype("Beast".into())],
        keywords: vec![KeywordAbility::Trample],
        power: Some(3),
        toughness: Some(2),
        mana_abilities: vec![],
        spell_effect: None,
        activated_abilities: vec![],
        triggered_abilities: vec![],
        starting_loyalty: None,
        static_abilities: vec![],
        enters_tapped: false,
        oracle_text: "Trample".into(),
        ..Default::default()
    });

    // --- Green spells ---

    // Rancor: G — Enchant creature gets +2/+0 and trample (simplified as buff)
    db.insert(CardDef {
        id: ids::RANCOR,
        name: "Rancor".into(),
        mana_cost: Some(ManaCost::new(0, 0, 0, 0, 0, 1)),
        card_types: vec![CardType::Enchantment],
        supertypes: vec![],
        subtypes: vec![Subtype("Aura".into())],
        keywords: vec![],
        power: None,
        toughness: None,
        mana_abilities: vec![],
        spell_effect: Some(Effect::Buff {
            power: 2,
            toughness: 0,
            until_eot: false,
        }),
        activated_abilities: vec![],
        triggered_abilities: vec![],
        starting_loyalty: None,
        static_abilities: vec![],
        enters_tapped: false,
        oracle_text: "Enchant creature. Enchanted creature gets +2/+0 and has trample. When Rancor is put into a graveyard from the battlefield, return Rancor to its owner's hand.".into(),
        ..Default::default()
    });

    // Vines of Vastwood: G — Target creature can't be the target of spells, +4/+4 if kicked (simplified as buff)
    db.insert(CardDef {
        id: ids::VINES_OF_VASTWOOD,
        name: "Vines of Vastwood".into(),
        mana_cost: Some(ManaCost::new(0, 0, 0, 0, 0, 1)),
        card_types: vec![CardType::Instant],
        supertypes: vec![],
        subtypes: vec![],
        keywords: vec![],
        power: None,
        toughness: None,
        mana_abilities: vec![],
        spell_effect: Some(Effect::Buff {
            power: 4,
            toughness: 4,
            until_eot: true,
        }),
        activated_abilities: vec![],
        triggered_abilities: vec![],
        starting_loyalty: None,
        static_abilities: vec![],
        enters_tapped: false,
        oracle_text: "Kicker {G}. Target creature can't be the target of spells or abilities your opponents control this turn. If Vines of Vastwood was kicked, that creature gets +4/+4 until end of turn.".into(),
        ..Default::default()
    });

    // Collected Company: 3G — Look at top 6, put up to 2 creatures with MV 3 or less onto battlefield (simplified as draw 2)
    db.insert(CardDef {
        id: ids::COLLECTED_COMPANY,
        name: "Collected Company".into(),
        mana_cost: Some(ManaCost::new(3, 0, 0, 0, 0, 1)),
        card_types: vec![CardType::Instant],
        supertypes: vec![],
        subtypes: vec![],
        keywords: vec![],
        power: None,
        toughness: None,
        mana_abilities: vec![],
        spell_effect: Some(Effect::DrawCards { count: 2 }),
        activated_abilities: vec![],
        triggered_abilities: vec![],
        starting_loyalty: None,
        static_abilities: vec![],
        enters_tapped: false,
        oracle_text: "Look at the top six cards of your library. Put up to two creature cards with mana value 3 or less from among them onto the battlefield. Put the rest on the bottom of your library in any random order.".into(),
        ..Default::default()
    });

    // Gaea's Anthem: 1GG — Creatures you control get +1/+1
    db.insert(CardDef {
        id: ids::GAEA_ANTHEM,
        name: "Gaea's Anthem".into(),
        mana_cost: Some(ManaCost::new(1, 0, 0, 0, 0, 2)),
        card_types: vec![CardType::Enchantment],
        supertypes: vec![],
        subtypes: vec![],
        keywords: vec![],
        power: None,
        toughness: None,
        mana_abilities: vec![],
        spell_effect: None,
        activated_abilities: vec![],
        triggered_abilities: vec![],
        starting_loyalty: None,
        static_abilities: vec![StaticAbility::Anthem {
            power: 1,
            toughness: 1,
            affected: AffectedObjects::OtherCreaturesControlledBy(0),
        }],
        enters_tapped: false,
        oracle_text: "Creatures you control get +1/+1.".into(),
        ..Default::default()
    });

    // --- Artifacts ---

    // Sol Ring: 1 — {T}: Add {C}{C} (simplified as mana ability)
    db.insert(CardDef {
        id: ids::SOL_RING,
        name: "Sol Ring".into(),
        mana_cost: Some(ManaCost::new(1, 0, 0, 0, 0, 0)),
        card_types: vec![CardType::Artifact],
        supertypes: vec![],
        subtypes: vec![],
        keywords: vec![],
        power: None,
        toughness: None,
        mana_abilities: vec![ManaAbility::TapForColorlessAmount(2)],
        spell_effect: None,
        activated_abilities: vec![],
        triggered_abilities: vec![],
        starting_loyalty: None,
        static_abilities: vec![],
        enters_tapped: false,
        oracle_text: "{T}: Add {C}{C}.".into(),
        ..Default::default()
    });

    // Signal Pest: 1 — 0/1 Artifact Creature — Flying
    db.insert(CardDef {
        id: ids::SIGNAL_PEST,
        name: "Signal Pest".into(),
        mana_cost: Some(ManaCost::new(1, 0, 0, 0, 0, 0)),
        card_types: vec![CardType::Artifact, CardType::Creature],
        supertypes: vec![],
        subtypes: vec![Subtype("Pest".into())],
        keywords: vec![KeywordAbility::Flying],
        power: Some(0),
        toughness: Some(1),
        mana_abilities: vec![],
        spell_effect: None,
        activated_abilities: vec![],
        triggered_abilities: vec![],
        starting_loyalty: None,
        static_abilities: vec![],
        enters_tapped: false,
        oracle_text:
            "Battle cry. Signal Pest can't be blocked except by creatures with flying or reach."
                .into(),
        ..Default::default()
    });

    // Vault Skirge: 1B/P — 1/1 Artifact Creature — Flying, lifelink
    db.insert(CardDef {
        id: ids::VAULT_SKIRGE,
        name: "Vault Skirge".into(),
        mana_cost: Some(ManaCost::new(1, 0, 0, 1, 0, 0)),
        card_types: vec![CardType::Artifact, CardType::Creature],
        supertypes: vec![],
        subtypes: vec![Subtype("Phyrexian".into()), Subtype("Imp".into())],
        keywords: vec![KeywordAbility::Flying, KeywordAbility::Lifelink],
        power: Some(1),
        toughness: Some(1),
        mana_abilities: vec![],
        spell_effect: None,
        activated_abilities: vec![],
        triggered_abilities: vec![],
        starting_loyalty: None,
        static_abilities: vec![],
        enters_tapped: false,
        oracle_text: "Flying, lifelink".into(),
        ..Default::default()
    });

    // Cranial Plating: 2 — Equipment artifact (simplified as buff enchantment)
    db.insert(CardDef {
        id: ids::CRANIAL_PLATING,
        name: "Cranial Plating".into(),
        mana_cost: Some(ManaCost::new(2, 0, 0, 0, 0, 0)),
        card_types: vec![CardType::Artifact],
        supertypes: vec![],
        subtypes: vec![Subtype("Equipment".into())],
        keywords: vec![],
        power: None,
        toughness: None,
        mana_abilities: vec![],
        spell_effect: None,
        activated_abilities: vec![],
        triggered_abilities: vec![],
        starting_loyalty: None,
        static_abilities: vec![],
        enters_tapped: false,
        oracle_text: "Equipped creature gets +1/+0 for each artifact you control. {B}{B}: Attach Cranial Plating to target creature you control. Equip {1}.".into(),
        ..Default::default()
    });

    // Steel Overseer: 2 — 1/1 Artifact Creature
    db.insert(CardDef {
        id: ids::STEEL_OVERSEER,
        name: "Steel Overseer".into(),
        mana_cost: Some(ManaCost::new(2, 0, 0, 0, 0, 0)),
        card_types: vec![CardType::Artifact, CardType::Creature],
        supertypes: vec![],
        subtypes: vec![Subtype("Construct".into())],
        keywords: vec![],
        power: Some(1),
        toughness: Some(1),
        mana_abilities: vec![],
        spell_effect: None,
        activated_abilities: vec![],
        triggered_abilities: vec![],
        starting_loyalty: None,
        static_abilities: vec![],
        enters_tapped: false,
        oracle_text: "{T}: Put a +1/+1 counter on each artifact creature you control.".into(),
        ..Default::default()
    });

    // --- Multicolor ---

    // Lightning Helix: RW — Deal 3, gain 3 life
    db.insert(CardDef {
        id: ids::LIGHTNING_HELIX,
        name: "Lightning Helix".into(),
        mana_cost: Some(ManaCost::new(0, 1, 0, 0, 1, 0)),
        card_types: vec![CardType::Instant],
        supertypes: vec![],
        subtypes: vec![],
        keywords: vec![],
        power: None,
        toughness: None,
        mana_abilities: vec![],
        spell_effect: Some(Effect::Multiple(vec![
            Effect::DealDamage {
                amount: 3,
                target: TargetSpec::CreatureOrPlayer,
            },
            Effect::GainLife { amount: 3 },
        ])),
        activated_abilities: vec![],
        triggered_abilities: vec![],
        starting_loyalty: None,
        static_abilities: vec![],
        enters_tapped: false,
        oracle_text: "Lightning Helix deals 3 damage to any target and you gain 3 life.".into(),
        ..Default::default()
    });

    // Terminate: BR — Destroy target creature
    db.insert(CardDef {
        id: ids::TERMINATE,
        name: "Terminate".into(),
        mana_cost: Some(ManaCost::new(0, 0, 0, 1, 1, 0)),
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
        activated_abilities: vec![],
        triggered_abilities: vec![],
        starting_loyalty: None,
        static_abilities: vec![],
        enters_tapped: false,
        oracle_text: "Destroy target creature. It can't be regenerated.".into(),
        ..Default::default()
    });

    // Geist of Saint Traft: 1WU 2/2 Spirit Cleric — Hexproof (simplified)
    db.insert(CardDef {
        id: ids::GEIST_OF_SAINT_TRAFT,
        name: "Geist of Saint Traft".into(),
        mana_cost: Some(ManaCost::new(1, 1, 1, 0, 0, 0)),
        card_types: vec![CardType::Creature],
        supertypes: vec![Supertype::Legendary],
        subtypes: vec![Subtype("Spirit".into()), Subtype("Cleric".into())],
        keywords: vec![],
        power: Some(2),
        toughness: Some(2),
        mana_abilities: vec![],
        spell_effect: None,
        activated_abilities: vec![],
        triggered_abilities: vec![],
        starting_loyalty: None,
        static_abilities: vec![],
        enters_tapped: false,
        oracle_text: "Hexproof. Whenever Geist of Saint Traft attacks, create a 4/4 white Angel creature token with flying that's tapped and attacking. Exile that token at end of combat.".into(),
        ..Default::default()
    });

    // Fleecemane Lion: GW 3/3 Cat
    db.insert(CardDef {
        id: ids::FLEECEMANE_LION,
        name: "Fleecemane Lion".into(),
        mana_cost: Some(ManaCost::new(0, 1, 0, 0, 0, 1)),
        card_types: vec![CardType::Creature],
        supertypes: vec![],
        subtypes: vec![Subtype("Cat".into())],
        keywords: vec![],
        power: Some(3),
        toughness: Some(3),
        mana_abilities: vec![],
        spell_effect: None,
        activated_abilities: vec![],
        triggered_abilities: vec![],
        starting_loyalty: None,
        static_abilities: vec![],
        enters_tapped: false,
        oracle_text: "{3}{G}{W}: Monstrosity 1. As long as Fleecemane Lion is monstrous, it has hexproof and indestructible.".into(),
        ..Default::default()
    });

    // Tidehollow Sculler: WB 2/2 Zombie — ETB exile a nonland card from opponent's hand (simplified as discard)
    db.insert(CardDef {
        id: ids::TIDEHOLLOW_SCULLER,
        name: "Tidehollow Sculler".into(),
        mana_cost: Some(ManaCost::new(0, 1, 0, 1, 0, 0)),
        card_types: vec![CardType::Artifact, CardType::Creature],
        supertypes: vec![],
        subtypes: vec![Subtype("Zombie".into())],
        keywords: vec![],
        power: Some(2),
        toughness: Some(2),
        mana_abilities: vec![],
        spell_effect: None,
        activated_abilities: vec![],
        triggered_abilities: vec![TriggeredAbility {
            trigger: TriggerCondition::EntersBattlefield,
            effect: Effect::DiscardCards {
                count: 1,
                target: TargetSpec::Opponent,
            },
            description: "When Tidehollow Sculler enters the battlefield, target opponent reveals their hand and you choose a nonland card from it. Exile that card.".into(),
        }],
        starting_loyalty: None,
        static_abilities: vec![],
        enters_tapped: false,
        oracle_text: "When Tidehollow Sculler enters the battlefield, target opponent reveals their hand and you choose a nonland card from it. Exile that card. When Tidehollow Sculler leaves the battlefield, return the exiled card to its owner's hand.".into(),
        ..Default::default()
    });

    // Dark Ritual: B — Add BBB
    db.insert(CardDef {
        id: ids::DARK_RITUAL,
        name: "Dark Ritual".into(),
        mana_cost: Some(ManaCost::new(0, 0, 0, 1, 0, 0)),
        card_types: vec![CardType::Instant],
        supertypes: vec![],
        subtypes: vec![],
        keywords: vec![],
        power: None,
        toughness: None,
        mana_abilities: vec![],
        spell_effect: Some(Effect::AddMana {
            color: Some(Color::Black),
            amount: 3,
        }),
        activated_abilities: vec![],
        triggered_abilities: vec![],
        starting_loyalty: None,
        static_abilities: vec![],
        enters_tapped: false,
        oracle_text: "Add {B}{B}{B}.".into(),
        ..Default::default()
    });

    // Stomping Ground: dual land (simplified as tapped forest)
    db.insert(CardDef {
        id: ids::STOMPING_GROUND,
        name: "Stomping Ground".into(),
        mana_cost: None,
        card_types: vec![CardType::Land],
        supertypes: vec![],
        subtypes: vec![Subtype("Mountain".into()), Subtype("Forest".into())],
        keywords: vec![],
        power: None,
        toughness: None,
        mana_abilities: vec![ManaAbility::TapForColor(Color::Red)],
        spell_effect: None,
        activated_abilities: vec![],
        triggered_abilities: vec![],
        starting_loyalty: None,
        static_abilities: vec![],
        enters_tapped: true,
        oracle_text: "As Stomping Ground enters the battlefield, you may pay 2 life. If you don't, it enters the battlefield tapped. {T}: Add {R} or {G}.".into(),
        ..Default::default()
    });

    // =====================================================================
    // Kinnan, Bonder Prodigy Commander Deck — Card definitions
    // =====================================================================

    // ---- Lands ----

    db.insert(CardDef {
        id: ids::ANCIENT_TOMB,
        name: "Ancient Tomb".into(),
        card_types: vec![CardType::Land],
        mana_abilities: vec![ManaAbility::TapForColorlessAmount(2)],
        oracle_text: "{T}: Add {C}{C}. Ancient Tomb deals 2 damage to you.".into(),
        ..Default::default()
    });

    db.insert(CardDef {
        id: ids::BOSEIJU_WHO_ENDURES,
        name: "Boseiju, Who Endures".into(),
        card_types: vec![CardType::Land],
        supertypes: vec![Supertype::Legendary],
        mana_abilities: vec![ManaAbility::TapForColor(Color::Green)],
        oracle_text: "{T}: Add {G}. Channel — {1}{G}, Discard Boseiju: Destroy target artifact, enchantment, or nonbasic land an opponent controls.".into(),
        ..Default::default()
    });

    db.insert(CardDef {
        id: ids::BREEDING_POOL,
        name: "Breeding Pool".into(),
        card_types: vec![CardType::Land],
        subtypes: vec![Subtype("Forest".into()), Subtype("Island".into())],
        mana_abilities: vec![ManaAbility::TapForChoice(vec![Color::Green, Color::Blue])],
        oracle_text: "({T}: Add {G} or {U}.) As Breeding Pool enters, you may pay 2 life. If you don't, it enters tapped.".into(),
        ..Default::default()
    });

    db.insert(CardDef {
        id: ids::COMMAND_TOWER,
        name: "Command Tower".into(),
        card_types: vec![CardType::Land],
        mana_abilities: vec![ManaAbility::TapForAny],
        oracle_text: "{T}: Add one mana of any color in your commander's color identity.".into(),
        ..Default::default()
    });

    db.insert(CardDef {
        id: ids::FLOODED_STRAND,
        name: "Flooded Strand".into(),
        card_types: vec![CardType::Land],
        activated_abilities: vec![ActivatedAbility {
            cost: ManaCost::zero(),
            requires_tap: true,
            effect: Effect::SearchLibrary { destination: ZoneType::Battlefield },
            description: "{T}, Pay 1 life, Sacrifice Flooded Strand: Search your library for a Plains or Island card, put it onto the battlefield, then shuffle.".into(),
        }],
        oracle_text: "{T}, Pay 1 life, Sacrifice Flooded Strand: Search your library for a Plains or Island card, put it onto the battlefield, then shuffle.".into(),
        ..Default::default()
    });

    db.insert(CardDef {
        id: ids::GAEAS_CRADLE,
        name: "Gaea's Cradle".into(),
        card_types: vec![CardType::Land],
        supertypes: vec![Supertype::Legendary],
        mana_abilities: vec![ManaAbility::TapForColor(Color::Green)],
        oracle_text: "{T}: Add {G} for each creature you control.".into(),
        ..Default::default()
    });

    db.insert(CardDef {
        id: ids::GEMSTONE_CAVERNS,
        name: "Gemstone Caverns".into(),
        card_types: vec![CardType::Land],
        supertypes: vec![Supertype::Legendary],
        mana_abilities: vec![ManaAbility::TapForAny],
        oracle_text: "If Gemstone Caverns is in your opening hand and you're not the starting player, you may begin the game with it on the battlefield with a luck counter on it. If you do, exile a card from your hand. {T}: Add {C}. If Gemstone Caverns has a luck counter on it, instead add one mana of any color.".into(),
        ..Default::default()
    });

    db.insert(CardDef {
        id: ids::INVENTORS_FAIR,
        name: "Inventors' Fair".into(),
        card_types: vec![CardType::Land],
        supertypes: vec![Supertype::Legendary],
        mana_abilities: vec![ManaAbility::TapForColorless],
        oracle_text: "At the beginning of your upkeep, if you control three or more artifacts, you gain 1 life. {T}: Add {C}. {4}, {T}, Sacrifice Inventors' Fair: Search your library for an artifact card, reveal it, put it into your hand, then shuffle. Activate only if you control three or more artifacts.".into(),
        ..Default::default()
    });

    db.insert(CardDef {
        id: ids::MINAMO_SCHOOL,
        name: "Minamo, School at Water's Edge".into(),
        card_types: vec![CardType::Land],
        supertypes: vec![Supertype::Legendary],
        mana_abilities: vec![ManaAbility::TapForColor(Color::Blue)],
        oracle_text: "{T}: Add {U}. {U}, {T}: Untap target legendary permanent.".into(),
        ..Default::default()
    });

    db.insert(CardDef {
        id: ids::MISTY_RAINFOREST,
        name: "Misty Rainforest".into(),
        card_types: vec![CardType::Land],
        activated_abilities: vec![ActivatedAbility {
            cost: ManaCost::zero(),
            requires_tap: true,
            effect: Effect::SearchLibrary { destination: ZoneType::Battlefield },
            description: "{T}, Pay 1 life, Sacrifice Misty Rainforest: Search your library for a Forest or Island card, put it onto the battlefield, then shuffle.".into(),
        }],
        oracle_text: "{T}, Pay 1 life, Sacrifice Misty Rainforest: Search your library for a Forest or Island card, put it onto the battlefield, then shuffle.".into(),
        ..Default::default()
    });

    db.insert(CardDef {
        id: ids::MISTRISE_VILLAGE,
        name: "Mistrise Village".into(),
        card_types: vec![CardType::Land],
        mana_abilities: vec![ManaAbility::TapForColor(Color::Blue)],
        oracle_text: "Mistrise Village enters tapped unless you control a Mountain or a Forest. {T}: Add {U}. {U}, {T}: The next spell you cast this turn can't be countered.".into(),
        ..Default::default()
    });

    db.insert(CardDef {
        id: ids::OTAWARA_SOARING_CITY,
        name: "Otawara, Soaring City".into(),
        card_types: vec![CardType::Land],
        supertypes: vec![Supertype::Legendary],
        mana_abilities: vec![ManaAbility::TapForColor(Color::Blue)],
        oracle_text: "{T}: Add {U}. Channel — {3}{U}, Discard Otawara: Return target artifact, creature, or planeswalker to its owner's hand.".into(),
        ..Default::default()
    });

    db.insert(CardDef {
        id: ids::SEAT_OF_THE_SYNOD,
        name: "Seat of the Synod".into(),
        card_types: vec![CardType::Artifact, CardType::Land],
        mana_abilities: vec![ManaAbility::TapForColor(Color::Blue)],
        oracle_text: "{T}: Add {U}.".into(),
        ..Default::default()
    });

    db.insert(CardDef {
        id: ids::SHIFTING_WOODLAND,
        name: "Shifting Woodland".into(),
        card_types: vec![CardType::Land],
        mana_abilities: vec![ManaAbility::TapForColor(Color::Green)],
        oracle_text: "Shifting Woodland enters tapped unless you control two or more other lands. {T}: Add {G}. Delirium — {2}{G}{G}: Shifting Woodland becomes a copy of target permanent card in your graveyard until end of turn.".into(),
        ..Default::default()
    });

    db.insert(CardDef {
        id: ids::SNOW_COVERED_FOREST,
        name: "Snow-Covered Forest".into(),
        card_types: vec![CardType::Land],
        supertypes: vec![Supertype::Basic, Supertype::Snow],
        subtypes: vec![Subtype("Forest".into())],
        mana_abilities: vec![ManaAbility::TapForColor(Color::Green)],
        oracle_text: "{T}: Add {G}.".into(),
        ..Default::default()
    });

    db.insert(CardDef {
        id: ids::SNOW_COVERED_ISLAND,
        name: "Snow-Covered Island".into(),
        card_types: vec![CardType::Land],
        supertypes: vec![Supertype::Basic, Supertype::Snow],
        subtypes: vec![Subtype("Island".into())],
        mana_abilities: vec![ManaAbility::TapForColor(Color::Blue)],
        oracle_text: "{T}: Add {U}.".into(),
        ..Default::default()
    });

    db.insert(CardDef {
        id: ids::TREASURE_VAULT,
        name: "Treasure Vault".into(),
        card_types: vec![CardType::Artifact, CardType::Land],
        mana_abilities: vec![ManaAbility::TapForColorless],
        oracle_text:
            "{T}: Add {C}. {X}{X}, {T}, Sacrifice Treasure Vault: Create X Treasure tokens.".into(),
        ..Default::default()
    });

    db.insert(CardDef {
        id: ids::TREE_OF_TALES,
        name: "Tree of Tales".into(),
        card_types: vec![CardType::Artifact, CardType::Land],
        mana_abilities: vec![ManaAbility::TapForColor(Color::Green)],
        oracle_text: "{T}: Add {G}.".into(),
        ..Default::default()
    });

    db.insert(CardDef {
        id: ids::TROPICAL_ISLAND,
        name: "Tropical Island".into(),
        card_types: vec![CardType::Land],
        subtypes: vec![Subtype("Forest".into()), Subtype("Island".into())],
        mana_abilities: vec![ManaAbility::TapForChoice(vec![Color::Green, Color::Blue])],
        oracle_text: "({T}: Add {G} or {U}.)".into(),
        ..Default::default()
    });

    db.insert(CardDef {
        id: ids::WATERLOGGED_GROVE,
        name: "Waterlogged Grove".into(),
        card_types: vec![CardType::Land],
        mana_abilities: vec![ManaAbility::TapForChoice(vec![Color::Green, Color::Blue])],
        oracle_text:
            "{T}, Pay 1 life: Add {G} or {U}. {1}, {T}, Sacrifice Waterlogged Grove: Draw a card."
                .into(),
        ..Default::default()
    });

    db.insert(CardDef {
        id: ids::WINDSWEPT_HEATH,
        name: "Windswept Heath".into(),
        card_types: vec![CardType::Land],
        activated_abilities: vec![ActivatedAbility {
            cost: ManaCost::zero(),
            requires_tap: true,
            effect: Effect::SearchLibrary { destination: ZoneType::Battlefield },
            description: "{T}, Pay 1 life, Sacrifice Windswept Heath: Search your library for a Forest or Plains card, put it onto the battlefield, then shuffle.".into(),
        }],
        oracle_text: "{T}, Pay 1 life, Sacrifice Windswept Heath: Search your library for a Forest or Plains card, put it onto the battlefield, then shuffle.".into(),
        ..Default::default()
    });

    db.insert(CardDef {
        id: ids::YAVIMAYA_COAST,
        name: "Yavimaya Coast".into(),
        card_types: vec![CardType::Land],
        mana_abilities: vec![ManaAbility::TapForChoice(vec![Color::Green, Color::Blue])],
        oracle_text: "{T}: Add {C}. {T}: Add {G} or {U}. Yavimaya Coast deals 1 damage to you."
            .into(),
        ..Default::default()
    });

    // ---- Artifacts ----

    db.insert(CardDef {
        id: ids::ARCANE_SIGNET,
        name: "Arcane Signet".into(),
        mana_cost: Some(ManaCost::new(2, 0, 0, 0, 0, 0)),
        card_types: vec![CardType::Artifact],
        mana_abilities: vec![ManaAbility::TapForAny],
        oracle_text: "{T}: Add one mana of any color in your commander's color identity.".into(),
        ..Default::default()
    });

    db.insert(CardDef {
        id: ids::BASALT_MONOLITH,
        name: "Basalt Monolith".into(),
        mana_cost: Some(ManaCost::new(3, 0, 0, 0, 0, 0)),
        card_types: vec![CardType::Artifact],
        mana_abilities: vec![ManaAbility::TapForColorlessAmount(3)],
        activated_abilities: vec![ActivatedAbility {
            cost: ManaCost::new(3, 0, 0, 0, 0, 0),
            requires_tap: false,
            effect: Effect::UntapTarget { target: TargetSpec::Controller },
            description: "{3}: Untap Basalt Monolith.".into(),
        }],
        oracle_text: "Basalt Monolith doesn't untap during your untap step. {T}: Add {C}{C}{C}. {3}: Untap Basalt Monolith.".into(),
        ..Default::default()
    });

    db.insert(CardDef {
        id: ids::CHROME_MOX,
        name: "Chrome Mox".into(),
        mana_cost: Some(ManaCost::zero()),
        card_types: vec![CardType::Artifact],
        mana_abilities: vec![ManaAbility::TapForAny],
        oracle_text: "Imprint — When Chrome Mox enters, you may exile a nonartifact, nonland card from your hand. {T}: Add one mana of any of the exiled card's colors.".into(),
        ..Default::default()
    });

    db.insert(CardDef {
        id: ids::FELLWAR_STONE,
        name: "Fellwar Stone".into(),
        mana_cost: Some(ManaCost::new(2, 0, 0, 0, 0, 0)),
        card_types: vec![CardType::Artifact],
        mana_abilities: vec![ManaAbility::TapForAny],
        oracle_text:
            "{T}: Add one mana of any color that a land an opponent controls could produce.".into(),
        ..Default::default()
    });

    db.insert(CardDef {
        id: ids::GRIM_MONOLITH,
        name: "Grim Monolith".into(),
        mana_cost: Some(ManaCost::new(2, 0, 0, 0, 0, 0)),
        card_types: vec![CardType::Artifact],
        mana_abilities: vec![ManaAbility::TapForColorlessAmount(3)],
        activated_abilities: vec![ActivatedAbility {
            cost: ManaCost::new(4, 0, 0, 0, 0, 0),
            requires_tap: false,
            effect: Effect::UntapTarget { target: TargetSpec::Controller },
            description: "{4}: Untap Grim Monolith.".into(),
        }],
        oracle_text: "Grim Monolith doesn't untap during your untap step. {T}: Add {C}{C}{C}. {4}: Untap Grim Monolith.".into(),
        ..Default::default()
    });

    db.insert(CardDef {
        id: ids::LOTUS_PETAL,
        name: "Lotus Petal".into(),
        mana_cost: Some(ManaCost::zero()),
        card_types: vec![CardType::Artifact],
        mana_abilities: vec![ManaAbility::TapForAny],
        oracle_text: "{T}, Sacrifice Lotus Petal: Add one mana of any color.".into(),
        ..Default::default()
    });

    db.insert(CardDef {
        id: ids::MANA_VAULT,
        name: "Mana Vault".into(),
        mana_cost: Some(ManaCost::new(1, 0, 0, 0, 0, 0)),
        card_types: vec![CardType::Artifact],
        mana_abilities: vec![ManaAbility::TapForColorlessAmount(3)],
        activated_abilities: vec![ActivatedAbility {
            cost: ManaCost::new(4, 0, 0, 0, 0, 0),
            requires_tap: false,
            effect: Effect::UntapTarget { target: TargetSpec::Controller },
            description: "{4}: Untap Mana Vault.".into(),
        }],
        oracle_text: "Mana Vault doesn't untap during your untap step. At the beginning of your upkeep, you may pay {4}. If you don't, Mana Vault deals 1 damage to you. {T}: Add {C}{C}{C}.".into(),
        ..Default::default()
    });

    db.insert(CardDef {
        id: ids::MOX_AMBER,
        name: "Mox Amber".into(),
        mana_cost: Some(ManaCost::zero()),
        card_types: vec![CardType::Artifact],
        supertypes: vec![Supertype::Legendary],
        mana_abilities: vec![ManaAbility::TapForAny],
        oracle_text: "{T}: Add one mana of any color among legendary creatures and planeswalkers you control.".into(),
        ..Default::default()
    });

    db.insert(CardDef {
        id: ids::MOX_DIAMOND,
        name: "Mox Diamond".into(),
        mana_cost: Some(ManaCost::zero()),
        card_types: vec![CardType::Artifact],
        mana_abilities: vec![ManaAbility::TapForAny],
        oracle_text: "If Mox Diamond would enter, you may discard a land card instead. If you do, put Mox Diamond onto the battlefield. If you don't, put it into its owner's graveyard. {T}: Add one mana of any color.".into(),
        ..Default::default()
    });

    db.insert(CardDef {
        id: ids::MOX_OPAL,
        name: "Mox Opal".into(),
        mana_cost: Some(ManaCost::zero()),
        card_types: vec![CardType::Artifact],
        supertypes: vec![Supertype::Legendary],
        mana_abilities: vec![ManaAbility::TapForAny],
        oracle_text: "Metalcraft — {T}: Add one mana of any color. Activate only if you control three or more artifacts.".into(),
        ..Default::default()
    });

    db.insert(CardDef {
        id: ids::MOONSILVER_KEY,
        name: "Moonsilver Key".into(),
        mana_cost: Some(ManaCost::new(2, 0, 0, 0, 0, 0)),
        card_types: vec![CardType::Artifact],
        spell_effect: Some(Effect::SearchLibrary { destination: ZoneType::Hand }),
        oracle_text: "{1}, {T}, Sacrifice Moonsilver Key: Search your library for an artifact card with a mana ability or a basic land card, reveal it, put it into your hand, then shuffle.".into(),
        ..Default::default()
    });

    db.insert(CardDef {
        id: ids::SIMIC_SIGNET,
        name: "Simic Signet".into(),
        mana_cost: Some(ManaCost::new(2, 0, 0, 0, 0, 0)),
        card_types: vec![CardType::Artifact],
        mana_abilities: vec![ManaAbility::TapForChoice(vec![Color::Green, Color::Blue])],
        oracle_text: "{1}, {T}: Add {G}{U}.".into(),
        ..Default::default()
    });

    db.insert(CardDef {
        id: ids::SPRINGLEAF_DRUM,
        name: "Springleaf Drum".into(),
        mana_cost: Some(ManaCost::new(1, 0, 0, 0, 0, 0)),
        card_types: vec![CardType::Artifact],
        mana_abilities: vec![ManaAbility::TapForAny],
        oracle_text: "{T}, Tap an untapped creature you control: Add one mana of any color.".into(),
        ..Default::default()
    });

    db.insert(CardDef {
        id: ids::TALISMAN_OF_CURIOSITY,
        name: "Talisman of Curiosity".into(),
        mana_cost: Some(ManaCost::new(2, 0, 0, 0, 0, 0)),
        card_types: vec![CardType::Artifact],
        mana_abilities: vec![ManaAbility::TapForChoice(vec![Color::Green, Color::Blue])],
        oracle_text: "{T}: Add {C}. {T}: Add {G} or {U}. Talisman of Curiosity deals 1 damage to you.".into(),
        ..Default::default()
    });

    db.insert(CardDef {
        id: ids::AGATHAS_SOUL_CAULDRON,
        name: "Agatha's Soul Cauldron".into(),
        mana_cost: Some(ManaCost::new(2, 0, 0, 0, 0, 0)),
        card_types: vec![CardType::Artifact],
        supertypes: vec![Supertype::Legendary],
        oracle_text: "You may spend mana as though it were mana of any color to activate abilities of creatures you control. {T}: Exile target card from a graveyard. When a creature card is exiled this way, put a +1/+1 counter on target creature you control. That creature gains all activated abilities of the exiled card.".into(),
        ..Default::default()
    });

    db.insert(CardDef {
        id: ids::THE_ONE_RING,
        name: "The One Ring".into(),
        mana_cost: Some(ManaCost::new(4, 0, 0, 0, 0, 0)),
        card_types: vec![CardType::Artifact],
        supertypes: vec![Supertype::Legendary],
        keywords: vec![KeywordAbility::Indestructible],
        activated_abilities: vec![ActivatedAbility {
            cost: ManaCost::new(1, 0, 0, 0, 0, 0),
            requires_tap: true,
            effect: Effect::DrawCards { count: 1 },
            description: "{1}, {T}: Put a burden counter on The One Ring, then draw a card for each burden counter on The One Ring.".into(),
        }],
        oracle_text: "Indestructible. When The One Ring enters, if you cast it, you gain protection from everything until your next turn. At the beginning of your upkeep, you lose 1 life for each burden counter on The One Ring. {T}: Put a burden counter on The One Ring, then draw a card for each burden counter on The One Ring.".into(),
        ..Default::default()
    });

    db.insert(CardDef {
        id: ids::MIRAGE_MIRROR,
        name: "Mirage Mirror".into(),
        mana_cost: Some(ManaCost::new(3, 0, 0, 0, 0, 0)),
        card_types: vec![CardType::Artifact],
        oracle_text: "{2}: Mirage Mirror becomes a copy of target artifact, creature, enchantment, or land until end of turn.".into(),
        ..Default::default()
    });

    // ---- Creatures ----

    db.insert(CardDef {
        id: ids::KINNAN_BONDER_PRODIGY,
        name: "Kinnan, Bonder Prodigy".into(),
        mana_cost: Some(ManaCost::new(0, 0, 1, 0, 0, 1)),
        card_types: vec![CardType::Creature],
        supertypes: vec![Supertype::Legendary],
        subtypes: vec![Subtype("Human".into()), Subtype("Druid".into())],
        power: Some(2),
        toughness: Some(2),
        static_abilities: vec![StaticAbility::ManaFromNonlandBonus],
        activated_abilities: vec![ActivatedAbility {
            cost: ManaCost::new(5, 0, 1, 0, 0, 1),
            requires_tap: false,
            effect: Effect::SearchLibrary { destination: ZoneType::Battlefield },
            description: "{5}{G}{U}: Look at the top five cards of your library. You may put a non-Human creature card from among them onto the battlefield. Put the rest on the bottom of your library in a random order.".into(),
        }],
        oracle_text: "Whenever you tap a nonland permanent for mana, add one mana of any type that permanent produced. {5}{G}{U}: Look at the top five cards of your library. You may put a non-Human creature card from among them onto the battlefield. Put the rest on the bottom in a random order.".into(),
        ..Default::default()
    });

    db.insert(CardDef {
        id: ids::BIRDS_OF_PARADISE,
        name: "Birds of Paradise".into(),
        mana_cost: Some(ManaCost::new(0, 0, 0, 0, 0, 1)),
        card_types: vec![CardType::Creature],
        subtypes: vec![Subtype("Bird".into())],
        keywords: vec![KeywordAbility::Flying],
        power: Some(0),
        toughness: Some(1),
        mana_abilities: vec![ManaAbility::TapForAny],
        oracle_text: "Flying. {T}: Add one mana of any color.".into(),
        ..Default::default()
    });

    db.insert(CardDef {
        id: ids::FYNDHORN_ELVES,
        name: "Fyndhorn Elves".into(),
        mana_cost: Some(ManaCost::new(0, 0, 0, 0, 0, 1)),
        card_types: vec![CardType::Creature],
        subtypes: vec![Subtype("Elf".into()), Subtype("Druid".into())],
        power: Some(1),
        toughness: Some(1),
        mana_abilities: vec![ManaAbility::TapForColor(Color::Green)],
        oracle_text: "{T}: Add {G}.".into(),
        ..Default::default()
    });

    db.insert(CardDef {
        id: ids::DELIGHTED_HALFLING,
        name: "Delighted Halfling".into(),
        mana_cost: Some(ManaCost::new(0, 0, 0, 0, 0, 1)),
        card_types: vec![CardType::Creature],
        subtypes: vec![Subtype("Halfling".into()), Subtype("Citizen".into())],
        power: Some(1),
        toughness: Some(2),
        mana_abilities: vec![ManaAbility::TapForColor(Color::Green), ManaAbility::TapForAny],
        oracle_text: "{T}: Add {G}. {T}: Add one mana of any color. Spend this mana only to cast a legendary spell, and that spell can't be countered.".into(),
        ..Default::default()
    });

    db.insert(CardDef {
        id: ids::BADGERMOLE_CUB,
        name: "Badgermole Cub".into(),
        mana_cost: Some(ManaCost::new(0, 0, 0, 0, 0, 2)),
        card_types: vec![CardType::Creature],
        subtypes: vec![Subtype("Badger".into()), Subtype("Mole".into())],
        power: Some(2),
        toughness: Some(2),
        oracle_text: "When this creature enters, earthbend 1. Whenever you tap a creature for mana, add an additional {G}.".into(),
        ..Default::default()
    });

    db.insert(CardDef {
        id: ids::CLEVER_IMPERSONATOR,
        name: "Clever Impersonator".into(),
        mana_cost: Some(ManaCost::new(2, 0, 2, 0, 0, 0)),
        card_types: vec![CardType::Creature],
        subtypes: vec![Subtype("Shapeshifter".into())],
        power: Some(0),
        toughness: Some(0),
        oracle_text: "You may have Clever Impersonator enter as a copy of any nonland permanent on the battlefield.".into(),
        ..Default::default()
    });

    db.insert(CardDef {
        id: ids::COLOSSAL_SKYTURTLE,
        name: "Colossal Skyturtle".into(),
        mana_cost: Some(ManaCost::new(5, 0, 1, 0, 0, 1)),
        card_types: vec![CardType::Creature],
        subtypes: vec![Subtype("Turtle".into())],
        keywords: vec![KeywordAbility::Flying],
        power: Some(6),
        toughness: Some(5),
        oracle_text: "Flying, ward {2}. Channel — {G}{U}, Discard Colossal Skyturtle: Return target creature to its owner's hand. Channel — {2}{G}, Discard Colossal Skyturtle: Return target permanent card from your graveyard to your hand.".into(),
        ..Default::default()
    });

    db.insert(CardDef {
        id: ids::CONSECRATED_SPHINX,
        name: "Consecrated Sphinx".into(),
        mana_cost: Some(ManaCost::new(4, 0, 2, 0, 0, 0)),
        card_types: vec![CardType::Creature],
        subtypes: vec![Subtype("Sphinx".into())],
        keywords: vec![KeywordAbility::Flying],
        power: Some(4),
        toughness: Some(6),
        triggered_abilities: vec![TriggeredAbility {
            trigger: TriggerCondition::OpponentDrawsCard,
            effect: Effect::DrawCards { count: 2 },
            description: "Whenever an opponent draws a card, you may draw two cards.".into(),
        }],
        oracle_text: "Flying. Whenever an opponent draws a card, you may draw two cards.".into(),
        ..Default::default()
    });

    db.insert(CardDef {
        id: ids::DRIFT_OF_PHANTASMS,
        name: "Drift of Phantasms".into(),
        mana_cost: Some(ManaCost::new(0, 0, 3, 0, 0, 0)),
        card_types: vec![CardType::Creature],
        subtypes: vec![Subtype("Spirit".into())],
        keywords: vec![KeywordAbility::Flying, KeywordAbility::Defender],
        power: Some(0),
        toughness: Some(5),
        oracle_text: "Flying, defender. Transmute {1}{U}{U}.".into(),
        ..Default::default()
    });

    db.insert(CardDef {
        id: ids::ELVISH_SPIRIT_GUIDE,
        name: "Elvish Spirit Guide".into(),
        mana_cost: Some(ManaCost::new(2, 0, 0, 0, 0, 1)),
        card_types: vec![CardType::Creature],
        subtypes: vec![Subtype("Elf".into()), Subtype("Spirit".into())],
        power: Some(2),
        toughness: Some(2),
        oracle_text: "Exile Elvish Spirit Guide from your hand: Add {G}.".into(),
        ..Default::default()
    });

    db.insert(CardDef {
        id: ids::ENDURANCE,
        name: "Endurance".into(),
        mana_cost: Some(ManaCost::new(1, 0, 0, 0, 0, 2)),
        card_types: vec![CardType::Creature],
        subtypes: vec![Subtype("Elemental".into()), Subtype("Incarnation".into())],
        keywords: vec![KeywordAbility::Flash, KeywordAbility::Reach],
        power: Some(3),
        toughness: Some(4),
        oracle_text: "Flash. Reach. When Endurance enters, up to one target player shuffles their graveyard into their library.".into(),
        ..Default::default()
    });

    db.insert(CardDef {
        id: ids::ENDURING_VITALITY,
        name: "Enduring Vitality".into(),
        mana_cost: Some(ManaCost::new(1, 0, 0, 0, 0, 2)),
        card_types: vec![CardType::Enchantment, CardType::Creature],
        subtypes: vec![Subtype("Elk".into()), Subtype("Glimmer".into())],
        keywords: vec![KeywordAbility::Vigilance],
        power: Some(3),
        toughness: Some(3),
        oracle_text: "Vigilance. Creatures you control have \"{T}: Add one mana of any color.\" When Enduring Vitality dies, if it was a creature, return it to the battlefield under its owner's control. It's an enchantment.".into(),
        ..Default::default()
    });

    db.insert(CardDef {
        id: ids::FAERIE_MASTERMIND,
        name: "Faerie Mastermind".into(),
        mana_cost: Some(ManaCost::new(1, 0, 1, 0, 0, 0)),
        card_types: vec![CardType::Creature],
        subtypes: vec![Subtype("Faerie".into()), Subtype("Rogue".into())],
        keywords: vec![KeywordAbility::Flash, KeywordAbility::Flying],
        power: Some(2),
        toughness: Some(1),
        triggered_abilities: vec![TriggeredAbility {
            trigger: TriggerCondition::OpponentDrawsCard,
            effect: Effect::DrawCards { count: 1 },
            description: "Whenever an opponent draws their second card each turn, you draw a card.".into(),
        }],
        activated_abilities: vec![ActivatedAbility {
            cost: ManaCost::new(3, 0, 1, 0, 0, 0),
            requires_tap: false,
            effect: Effect::DrawCards { count: 1 },
            description: "{3}{U}: Each player draws a card.".into(),
        }],
        oracle_text: "Flash. Flying. Whenever an opponent draws their second card each turn, you draw a card. {3}{U}: Each player draws a card.".into(),
        ..Default::default()
    });

    db.insert(CardDef {
        id: ids::FLESH_DUPLICATE,
        name: "Flesh Duplicate".into(),
        mana_cost: Some(ManaCost::new(1, 0, 1, 0, 0, 0)),
        card_types: vec![CardType::Creature],
        subtypes: vec![Subtype("Shapeshifter".into())],
        power: Some(0),
        toughness: Some(0),
        oracle_text: "You may have Flesh Duplicate enter as a copy of any creature on the battlefield, except it has vanishing 3.".into(),
        ..Default::default()
    });

    db.insert(CardDef {
        id: ids::HIGH_FAE_TRICKSTER,
        name: "High Fae Trickster".into(),
        mana_cost: Some(ManaCost::new(3, 0, 1, 0, 0, 0)),
        card_types: vec![CardType::Creature],
        subtypes: vec![Subtype("Faerie".into()), Subtype("Wizard".into())],
        keywords: vec![KeywordAbility::Flash, KeywordAbility::Flying],
        power: Some(4),
        toughness: Some(2),
        oracle_text: "Flash. Flying. You may cast spells as though they had flash.".into(),
        ..Default::default()
    });

    db.insert(CardDef {
        id: ids::HULLBREAKER_HORROR,
        name: "Hullbreaker Horror".into(),
        mana_cost: Some(ManaCost::new(5, 0, 2, 0, 0, 0)),
        card_types: vec![CardType::Creature],
        subtypes: vec![Subtype("Kraken".into()), Subtype("Horror".into())],
        keywords: vec![KeywordAbility::Flash],
        power: Some(7),
        toughness: Some(8),
        triggered_abilities: vec![TriggeredAbility {
            trigger: TriggerCondition::YouCastSpell,
            effect: Effect::BounceTo { zone: ZoneType::Hand, target: TargetSpec::AnyNonlandPermanent },
            description: "Whenever you cast a spell, choose up to one: return target nonland permanent to its owner's hand; or counter target spell.".into(),
        }],
        oracle_text: "Flash. This spell can't be countered. Whenever you cast a spell, choose up to one — Return target nonland permanent to its owner's hand; or counter target spell.".into(),
        ..Default::default()
    });

    db.insert(CardDef {
        id: ids::MOCKINGBIRD,
        name: "Mockingbird".into(),
        mana_cost: Some(ManaCost::new(1, 0, 1, 0, 0, 0)),
        card_types: vec![CardType::Creature],
        subtypes: vec![Subtype("Bird".into()), Subtype("Bard".into())],
        keywords: vec![KeywordAbility::Flying],
        power: Some(1),
        toughness: Some(1),
        oracle_text: "Flying. You may have Mockingbird enter as a copy of any creature on the battlefield with mana value less than or equal to the amount of mana spent to cast Mockingbird, except it's a Bird in addition to its other types and it has flying.".into(),
        ..Default::default()
    });

    db.insert(CardDef {
        id: ids::NEZAHAL_PRIMAL_TIDE,
        name: "Nezahal, Primal Tide".into(),
        mana_cost: Some(ManaCost::new(5, 0, 2, 0, 0, 0)),
        card_types: vec![CardType::Creature],
        supertypes: vec![Supertype::Legendary],
        subtypes: vec![Subtype("Elder".into()), Subtype("Dinosaur".into())],
        power: Some(7),
        toughness: Some(7),
        triggered_abilities: vec![TriggeredAbility {
            trigger: TriggerCondition::OpponentCastsNoncreatureSpell,
            effect: Effect::DrawCards { count: 1 },
            description: "Whenever an opponent casts a noncreature spell, draw a card.".into(),
        }],
        oracle_text: "This spell can't be countered. Whenever an opponent casts a noncreature spell, draw a card. Discard three cards: Exile Nezahal. Return it to the battlefield tapped under its owner's control at the beginning of the next end step.".into(),
        ..Default::default()
    });

    db.insert(CardDef {
        id: ids::NYXBLOOM_ANCIENT,
        name: "Nyxbloom Ancient".into(),
        mana_cost: Some(ManaCost::new(4, 0, 0, 0, 0, 3)),
        card_types: vec![CardType::Creature, CardType::Enchantment],
        subtypes: vec![Subtype("Elemental".into())],
        keywords: vec![KeywordAbility::Trample],
        power: Some(5),
        toughness: Some(5),
        oracle_text: "Trample. If you tap a permanent for mana, it produces three times as much of that mana instead.".into(),
        ..Default::default()
    });

    db.insert(CardDef {
        id: ids::PHYREXIAN_METAMORPH,
        name: "Phyrexian Metamorph".into(),
        mana_cost: Some(ManaCost::new(3, 0, 1, 0, 0, 0)),
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: vec![Subtype("Phyrexian".into()), Subtype("Shapeshifter".into())],
        power: Some(0),
        toughness: Some(0),
        oracle_text: "({U/P} can be paid with either {U} or 2 life.) You may have Phyrexian Metamorph enter as a copy of any artifact or creature on the battlefield, except it's an artifact in addition to its other types.".into(),
        ..Default::default()
    });

    db.insert(CardDef {
        id: ids::SEEDBORN_MUSE,
        name: "Seedborn Muse".into(),
        mana_cost: Some(ManaCost::new(3, 0, 0, 0, 0, 2)),
        card_types: vec![CardType::Creature],
        subtypes: vec![Subtype("Spirit".into())],
        power: Some(2),
        toughness: Some(4),
        oracle_text: "Untap all permanents you control during each other player's untap step."
            .into(),
        ..Default::default()
    });

    db.insert(CardDef {
        id: ids::THRASIOS_TRITON_HERO,
        name: "Thrasios, Triton Hero".into(),
        mana_cost: Some(ManaCost::new(0, 0, 1, 0, 0, 1)),
        card_types: vec![CardType::Creature],
        supertypes: vec![Supertype::Legendary],
        subtypes: vec![Subtype("Merfolk".into()), Subtype("Wizard".into())],
        power: Some(1),
        toughness: Some(3),
        activated_abilities: vec![ActivatedAbility {
            cost: ManaCost::new(4, 0, 0, 0, 0, 0),
            requires_tap: false,
            effect: Effect::DrawCards { count: 1 },
            description: "{4}: Scry 1, then reveal the top card of your library. If it's a land card, put it onto the battlefield tapped. Otherwise, draw a card.".into(),
        }],
        oracle_text: "Partner. {4}: Scry 1, then reveal the top card of your library. If it's a land card, put it onto the battlefield tapped. Otherwise, draw a card.".into(),
        ..Default::default()
    });

    db.insert(CardDef {
        id: ids::TIDESPOUT_TYRANT,
        name: "Tidespout Tyrant".into(),
        mana_cost: Some(ManaCost::new(5, 0, 3, 0, 0, 0)),
        card_types: vec![CardType::Creature],
        subtypes: vec![Subtype("Djinn".into())],
        keywords: vec![KeywordAbility::Flying],
        power: Some(5),
        toughness: Some(5),
        triggered_abilities: vec![TriggeredAbility {
            trigger: TriggerCondition::YouCastSpell,
            effect: Effect::BounceTo { zone: ZoneType::Hand, target: TargetSpec::AnyPermanent },
            description: "Whenever you cast a spell, return target permanent to its owner's hand.".into(),
        }],
        oracle_text: "Flying. Whenever you cast a spell, return target permanent to its owner's hand.".into(),
        ..Default::default()
    });

    db.insert(CardDef {
        id: ids::TROPHY_MAGE,
        name: "Trophy Mage".into(),
        mana_cost: Some(ManaCost::new(2, 0, 1, 0, 0, 0)),
        card_types: vec![CardType::Creature],
        subtypes: vec![Subtype("Human".into()), Subtype("Wizard".into())],
        power: Some(2),
        toughness: Some(2),
        triggered_abilities: vec![TriggeredAbility {
            trigger: TriggerCondition::EntersBattlefield,
            effect: Effect::SearchLibrary { destination: ZoneType::Hand },
            description: "When Trophy Mage enters, you may search your library for an artifact card with mana value 3, reveal it, put it into your hand, then shuffle.".into(),
        }],
        oracle_text: "When Trophy Mage enters, you may search your library for an artifact card with mana value 3, reveal it, put it into your hand, then shuffle.".into(),
        ..Default::default()
    });

    db.insert(CardDef {
        id: ids::WAN_SHI_TONG,
        name: "Wan Shi Tong, Librarian".into(),
        mana_cost: Some(ManaCost::new(2, 0, 2, 0, 0, 0)),
        card_types: vec![CardType::Creature],
        supertypes: vec![Supertype::Legendary],
        subtypes: vec![Subtype("Bird".into()), Subtype("Spirit".into())],
        keywords: vec![KeywordAbility::Flash, KeywordAbility::Flying, KeywordAbility::Vigilance],
        power: Some(1),
        toughness: Some(1),
        triggered_abilities: vec![TriggeredAbility {
            trigger: TriggerCondition::EntersBattlefield,
            effect: Effect::DrawCards { count: 1 },
            description: "When Wan Shi Tong enters, put X +1/+1 counters on him. Then draw half X cards, rounded down.".into(),
        }],
        oracle_text: "Flash, flying, vigilance. When Wan Shi Tong enters, put X +1/+1 counters on him. Then draw half X cards, rounded down. Whenever an opponent searches their library, put a +1/+1 counter on Wan Shi Tong and draw a card.".into(),
        ..Default::default()
    });

    db.insert(CardDef {
        id: ids::WANDERING_ARCHAIC,
        name: "Wandering Archaic // Explore the Vastlands".into(),
        mana_cost: Some(ManaCost::new(5, 0, 0, 0, 0, 0)),
        card_types: vec![CardType::Creature],
        subtypes: vec![Subtype("Avatar".into())],
        power: Some(4),
        toughness: Some(4),
        oracle_text: "Whenever an opponent casts an instant or sorcery spell, they may pay {2}. If they don't, you may copy that spell. You may choose new targets for the copy.".into(),
        ..Default::default()
    });

    // ---- Instants ----

    db.insert(CardDef {
        id: ids::AN_OFFER_YOU_CANT_REFUSE,
        name: "An Offer You Can't Refuse".into(),
        mana_cost: Some(ManaCost::new(0, 0, 1, 0, 0, 0)),
        card_types: vec![CardType::Instant],
        spell_effect: Some(Effect::Counter {
            target: TargetSpec::AnySpell,
        }),
        oracle_text:
            "Counter target noncreature spell. Its controller creates two Treasure tokens.".into(),
        ..Default::default()
    });

    db.insert(CardDef {
        id: ids::CHORD_OF_CALLING,
        name: "Chord of Calling".into(),
        mana_cost: Some(ManaCost::new(3, 0, 0, 0, 0, 3)),
        card_types: vec![CardType::Instant],
        spell_effect: Some(Effect::SearchLibrary { destination: ZoneType::Battlefield }),
        oracle_text: "Convoke. Search your library for a creature card with mana value X or less, put it onto the battlefield, then shuffle.".into(),
        ..Default::default()
    });

    db.insert(CardDef {
        id: ids::CROP_ROTATION,
        name: "Crop Rotation".into(),
        mana_cost: Some(ManaCost::new(0, 0, 0, 0, 0, 1)),
        card_types: vec![CardType::Instant],
        spell_effect: Some(Effect::SearchLibrary { destination: ZoneType::Battlefield }),
        oracle_text: "As an additional cost to cast this spell, sacrifice a land. Search your library for a land card, put that card onto the battlefield, then shuffle.".into(),
        ..Default::default()
    });

    db.insert(CardDef {
        id: ids::CYCLONIC_RIFT,
        name: "Cyclonic Rift".into(),
        mana_cost: Some(ManaCost::new(1, 0, 1, 0, 0, 0)),
        card_types: vec![CardType::Instant],
        spell_effect: Some(Effect::BounceTo { zone: ZoneType::Hand, target: TargetSpec::AnyNonlandPermanent }),
        oracle_text: "Return target nonland permanent you don't control to its owner's hand. Overload {6}{U} (You may cast this spell for its overload cost. If you do, change 'target' to 'each'.)".into(),
        ..Default::default()
    });

    db.insert(CardDef {
        id: ids::FIERCE_GUARDIANSHIP,
        name: "Fierce Guardianship".into(),
        mana_cost: Some(ManaCost::new(2, 0, 1, 0, 0, 0)),
        card_types: vec![CardType::Instant],
        spell_effect: Some(Effect::Counter { target: TargetSpec::AnySpell }),
        oracle_text: "If you control a commander, you may cast this spell without paying its mana cost. Counter target noncreature spell.".into(),
        ..Default::default()
    });

    db.insert(CardDef {
        id: ids::FLUSTERSTORM,
        name: "Flusterstorm".into(),
        mana_cost: Some(ManaCost::new(0, 0, 1, 0, 0, 0)),
        card_types: vec![CardType::Instant],
        spell_effect: Some(Effect::Counter {
            target: TargetSpec::AnySpell,
        }),
        oracle_text:
            "Counter target instant or sorcery spell unless its controller pays {1}. Storm.".into(),
        ..Default::default()
    });

    db.insert(CardDef {
        id: ids::FORCE_OF_NEGATION,
        name: "Force of Negation".into(),
        mana_cost: Some(ManaCost::new(1, 0, 2, 0, 0, 0)),
        card_types: vec![CardType::Instant],
        spell_effect: Some(Effect::Counter { target: TargetSpec::AnySpell }),
        oracle_text: "If it's not your turn, you may exile a blue card from your hand rather than pay this spell's mana cost. Counter target noncreature spell. If that spell is countered this way, exile it instead of putting it into its owner's graveyard.".into(),
        ..Default::default()
    });

    db.insert(CardDef {
        id: ids::FORCE_OF_WILL,
        name: "Force of Will".into(),
        mana_cost: Some(ManaCost::new(3, 0, 2, 0, 0, 0)),
        card_types: vec![CardType::Instant],
        spell_effect: Some(Effect::Counter { target: TargetSpec::AnySpell }),
        oracle_text: "You may pay 1 life and exile a blue card from your hand rather than pay this spell's mana cost. Counter target spell.".into(),
        ..Default::default()
    });

    db.insert(CardDef {
        id: ids::INTO_THE_FLOOD_MAW,
        name: "Into the Flood Maw".into(),
        mana_cost: Some(ManaCost::new(0, 0, 1, 0, 0, 0)),
        card_types: vec![CardType::Instant],
        spell_effect: Some(Effect::BounceTo { zone: ZoneType::Hand, target: TargetSpec::AnyCreature }),
        oracle_text: "Gift a tapped Fish. Return target creature an opponent controls to its owner's hand. If the gift was promised, instead return target nonland permanent an opponent controls to its owner's hand.".into(),
        ..Default::default()
    });

    db.insert(CardDef {
        id: ids::MENTAL_MISSTEP,
        name: "Mental Misstep".into(),
        mana_cost: Some(ManaCost::new(0, 0, 1, 0, 0, 0)),
        card_types: vec![CardType::Instant],
        spell_effect: Some(Effect::Counter {
            target: TargetSpec::AnySpell,
        }),
        oracle_text:
            "({U/P} can be paid with either {U} or 2 life.) Counter target spell with mana value 1."
                .into(),
        ..Default::default()
    });

    db.insert(CardDef {
        id: ids::MINDBREAK_TRAP,
        name: "Mindbreak Trap".into(),
        mana_cost: Some(ManaCost::new(2, 0, 2, 0, 0, 0)),
        card_types: vec![CardType::Instant],
        spell_effect: Some(Effect::Counter { target: TargetSpec::AnySpell }),
        oracle_text: "If an opponent cast three or more spells this turn, you may pay {0} rather than pay this spell's mana cost. Exile any number of target spells.".into(),
        ..Default::default()
    });

    db.insert(CardDef {
        id: ids::MYSTICAL_TUTOR,
        name: "Mystical Tutor".into(),
        mana_cost: Some(ManaCost::new(0, 0, 1, 0, 0, 0)),
        card_types: vec![CardType::Instant],
        spell_effect: Some(Effect::SearchLibrary { destination: ZoneType::Library }),
        oracle_text: "Search your library for an instant or sorcery card, reveal it, then shuffle and put that card on top.".into(),
        ..Default::default()
    });

    db.insert(CardDef {
        id: ids::NOXIOUS_REVIVAL,
        name: "Noxious Revival".into(),
        mana_cost: Some(ManaCost::new(0, 0, 0, 0, 0, 1)),
        card_types: vec![CardType::Instant],
        spell_effect: Some(Effect::ReturnToTopOfLibrary { target: TargetSpec::NoTarget }),
        oracle_text: "({G/P} can be paid with either {G} or 2 life.) Put target card from a graveyard on top of its owner's library.".into(),
        ..Default::default()
    });

    db.insert(CardDef {
        id: ids::PACT_OF_NEGATION,
        name: "Pact of Negation".into(),
        mana_cost: Some(ManaCost::zero()),
        card_types: vec![CardType::Instant],
        spell_effect: Some(Effect::Counter { target: TargetSpec::AnySpell }),
        oracle_text: "Counter target spell. At the beginning of your next upkeep, pay {3}{U}{U}. If you don't, you lose the game.".into(),
        ..Default::default()
    });

    db.insert(CardDef {
        id: ids::SWAN_SONG,
        name: "Swan Song".into(),
        mana_cost: Some(ManaCost::new(0, 0, 1, 0, 0, 0)),
        card_types: vec![CardType::Instant],
        spell_effect: Some(Effect::Counter { target: TargetSpec::AnySpell }),
        oracle_text: "Counter target enchantment, instant, or sorcery spell. Its controller creates a 2/2 blue Bird creature token with flying.".into(),
        ..Default::default()
    });

    db.insert(CardDef {
        id: ids::VEIL_OF_SUMMER,
        name: "Veil of Summer".into(),
        mana_cost: Some(ManaCost::new(0, 0, 0, 0, 0, 1)),
        card_types: vec![CardType::Instant],
        spell_effect: Some(Effect::DrawCards { count: 1 }),
        oracle_text: "Draw a card if an opponent has cast a blue or black spell this turn. Spells you control can't be countered this turn. You and permanents you control gain hexproof from blue and from black until end of turn.".into(),
        ..Default::default()
    });

    db.insert(CardDef {
        id: ids::WHIR_OF_INVENTION,
        name: "Whir of Invention".into(),
        mana_cost: Some(ManaCost::new(3, 0, 3, 0, 0, 0)),
        card_types: vec![CardType::Instant],
        spell_effect: Some(Effect::SearchLibrary { destination: ZoneType::Battlefield }),
        oracle_text: "Improvise. Search your library for an artifact card with mana value X or less, put it onto the battlefield, then shuffle.".into(),
        ..Default::default()
    });

    db.insert(CardDef {
        id: ids::WORLDLY_TUTOR,
        name: "Worldly Tutor".into(),
        mana_cost: Some(ManaCost::new(0, 0, 0, 0, 0, 1)),
        card_types: vec![CardType::Instant],
        spell_effect: Some(Effect::SearchLibrary { destination: ZoneType::Library }),
        oracle_text: "Search your library for a creature card, reveal it, then shuffle and put that card on top.".into(),
        ..Default::default()
    });

    db.insert(CardDef {
        id: ids::MUDDLE_THE_MIXTURE,
        name: "Muddle the Mixture".into(),
        mana_cost: Some(ManaCost::new(0, 0, 2, 0, 0, 0)),
        card_types: vec![CardType::Instant],
        spell_effect: Some(Effect::Counter {
            target: TargetSpec::AnySpell,
        }),
        oracle_text: "Counter target instant or sorcery spell. Transmute {1}{U}{U}.".into(),
        ..Default::default()
    });

    // ---- Sorceries ----

    db.insert(CardDef {
        id: ids::FINALE_OF_DEVASTATION,
        name: "Finale of Devastation".into(),
        mana_cost: Some(ManaCost::new(2, 0, 0, 0, 0, 2)),
        card_types: vec![CardType::Sorcery],
        spell_effect: Some(Effect::SearchLibrary { destination: ZoneType::Battlefield }),
        oracle_text: "Search your library and/or graveyard for a creature card with mana value X or less and put it onto the battlefield. If you search your library this way, shuffle. If X is 10 or more, creatures you control get +X/+X and gain haste until end of turn.".into(),
        ..Default::default()
    });

    db.insert(CardDef {
        id: ids::GREEN_SUNS_ZENITH,
        name: "Green Sun's Zenith".into(),
        mana_cost: Some(ManaCost::new(1, 0, 0, 0, 0, 1)),
        card_types: vec![CardType::Sorcery],
        spell_effect: Some(Effect::SearchLibrary { destination: ZoneType::Battlefield }),
        oracle_text: "Search your library for a green creature card with mana value X or less, put it onto the battlefield, then shuffle. Shuffle Green Sun's Zenith into its owner's library.".into(),
        ..Default::default()
    });

    db.insert(CardDef {
        id: ids::NATURES_RHYTHM,
        name: "Nature's Rhythm".into(),
        mana_cost: Some(ManaCost::new(1, 0, 0, 0, 0, 2)),
        card_types: vec![CardType::Sorcery],
        spell_effect: Some(Effect::SearchLibrary { destination: ZoneType::Battlefield }),
        oracle_text: "Search your library for a creature card with mana value X or less, put it onto the battlefield, then shuffle. Harmonize {X}{G}{G}{G}{G}.".into(),
        ..Default::default()
    });

    // ---- Enchantments ----

    db.insert(CardDef {
        id: ids::MYSTIC_REMORA,
        name: "Mystic Remora".into(),
        mana_cost: Some(ManaCost::new(0, 0, 1, 0, 0, 0)),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![TriggeredAbility {
            trigger: TriggerCondition::OpponentCastsNoncreatureSpell,
            effect: Effect::DrawCards { count: 1 },
            description: "Whenever an opponent casts a noncreature spell, you may draw a card unless that player pays {4}.".into(),
        }],
        oracle_text: "Cumulative upkeep {1}. Whenever an opponent casts a noncreature spell, you may draw a card unless that player pays {4}.".into(),
        ..Default::default()
    });

    db.insert(CardDef {
        id: ids::RHYSTIC_STUDY,
        name: "Rhystic Study".into(),
        mana_cost: Some(ManaCost::new(2, 0, 1, 0, 0, 0)),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![TriggeredAbility {
            trigger: TriggerCondition::OpponentCastsSpell,
            effect: Effect::DrawCards { count: 1 },
            description: "Whenever an opponent casts a spell, you may draw a card unless that player pays {1}.".into(),
        }],
        oracle_text: "Whenever an opponent casts a spell, you may draw a card unless that player pays {1}.".into(),
        ..Default::default()
    });

    // ---- Planeswalkers ----

    db.insert(CardDef {
        id: ids::TEZZERET_THE_SEEKER,
        name: "Tezzeret the Seeker".into(),
        mana_cost: Some(ManaCost::new(3, 0, 2, 0, 0, 0)),
        card_types: vec![CardType::Planeswalker],
        subtypes: vec![Subtype("Tezzeret".into())],
        starting_loyalty: Some(4),
        oracle_text: "+1: Untap up to two target artifacts. -X: Search your library for an artifact card with mana value X or less, put it onto the battlefield, then shuffle. -5: Artifacts you control become artifact creatures with base power and toughness 5/5 until end of turn.".into(),
        ..Default::default()
    });

    // ---- DFC / Battle cards ----

    // Bridgeworks Battle // Tanglespan Bridgeworks (front face: sorcery)
    db.insert(CardDef {
        id: ids::BRIDGEWORKS_BATTLE,
        name: "Bridgeworks Battle // Tanglespan Bridgeworks".into(),
        mana_cost: Some(ManaCost::new(2, 0, 0, 0, 0, 1)),
        card_types: vec![CardType::Sorcery],
        spell_effect: Some(Effect::Multiple(vec![
            Effect::Buff { power: 2, toughness: 2, until_eot: true },
        ])),
        oracle_text: "Target creature you control gets +2/+2 until end of turn. It fights up to one target creature you don't control. // Tanglespan Bridgeworks — Land. As this enters, you may pay 3 life. If you don't, it enters tapped. {T}: Add {G}.".into(),
        ..Default::default()
    });

    // Disciple of Freyalise // Garden of Freyalise (front face: creature)
    db.insert(CardDef {
        id: ids::DISCIPLE_OF_FREYALISE,
        name: "Disciple of Freyalise // Garden of Freyalise".into(),
        mana_cost: Some(ManaCost::new(3, 0, 0, 0, 0, 3)),
        card_types: vec![CardType::Creature],
        subtypes: vec![Subtype("Elf".into()), Subtype("Druid".into())],
        power: Some(3),
        toughness: Some(3),
        triggered_abilities: vec![TriggeredAbility {
            trigger: TriggerCondition::EntersBattlefield,
            effect: Effect::DrawCards { count: 1 },
            description: "When Disciple of Freyalise enters, you may sacrifice another creature. If you do, you gain X life and draw X cards, where X is that creature's power.".into(),
        }],
        oracle_text: "When Disciple of Freyalise enters, you may sacrifice another creature. If you do, you gain X life and draw X cards, where X is that creature's power. // Garden of Freyalise — Land. As this enters, you may pay 3 life. If you don't, it enters tapped. {T}: Add {G}.".into(),
        ..Default::default()
    });

    // Hydroelectric Specimen // Hydroelectric Laboratory (front face: creature)
    db.insert(CardDef {
        id: ids::HYDROELECTRIC_SPECIMEN,
        name: "Hydroelectric Specimen // Hydroelectric Laboratory".into(),
        mana_cost: Some(ManaCost::new(2, 0, 1, 0, 0, 0)),
        card_types: vec![CardType::Creature],
        subtypes: vec![Subtype("Weird".into())],
        keywords: vec![KeywordAbility::Flash],
        power: Some(1),
        toughness: Some(4),
        oracle_text: "Flash. When this creature enters, you may change the target of target instant or sorcery spell with a single target to this creature. // Hydroelectric Laboratory — Land. Enters tapped. {T}: Add {U}.".into(),
        ..Default::default()
    });

    // Invasion of Ikoria // Zilortha, Apex of Ikoria (battle front face)
    db.insert(CardDef {
        id: ids::INVASION_OF_IKORIA,
        name: "Invasion of Ikoria // Zilortha, Apex of Ikoria".into(),
        mana_cost: Some(ManaCost::new(2, 0, 0, 0, 0, 2)),
        card_types: vec![CardType::Sorcery],
        spell_effect: Some(Effect::SearchLibrary { destination: ZoneType::Hand }),
        oracle_text: "When Invasion of Ikoria enters, search your library for a non-Human creature card with mana value X or less, reveal it, put it into your hand, then shuffle. // Zilortha, Apex of Ikoria — 7/3 Dinosaur. Trample. Each creature you control with power greater than its toughness assigns combat damage equal to its power rather than its toughness.".into(),
        ..Default::default()
    });

    // Sink into Stupor // Soporific Springs (front face: instant)
    db.insert(CardDef {
        id: ids::SINK_INTO_STUPOR,
        name: "Sink into Stupor // Soporific Springs".into(),
        mana_cost: Some(ManaCost::new(1, 0, 1, 1, 0, 0)),
        card_types: vec![CardType::Instant],
        spell_effect: Some(Effect::BounceTo { zone: ZoneType::Hand, target: TargetSpec::AnyNonlandPermanent }),
        oracle_text: "Choose one — Target opponent discards two cards; or return target nonland permanent to its owner's hand. // Soporific Springs — Land. Enters tapped. {T}: Add {U} or {B}.".into(),
        ..Default::default()
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

// ==========================================================================
// Commander decks (100-card singleton, except basics)
// ==========================================================================

/// Build a mono-white Commander deck with Brimaz, King of Oreskos as
/// the commander. Uses all available white cards from the pool plus
/// colorless artifacts, padded with Plains to reach 100 cards.
///
/// Returns (deck, commander_id).
pub fn brimaz_commander_deck() -> (Vec<CardId>, CardId) {
    let commander = ids::BRIMAZ_KING;
    let mut deck = Vec::new();

    // Commander (included in the deck list, will be extracted by setup)
    deck.push(commander);

    // White creatures (1 each, singleton)
    deck.push(ids::SAVANNAH_LIONS);
    deck.push(ids::MOTHER_OF_RUNES);
    deck.push(ids::ELITE_VANGUARD);
    deck.push(ids::WHITE_KNIGHT);
    deck.push(ids::LEONIN_SKYHUNTER);
    deck.push(ids::BANESLAYER_ANGEL);
    deck.push(ids::SERRA_ANGEL);
    deck.push(ids::THALIA_GUARDIAN);
    deck.push(ids::SOLDIER_OF_THE_PANTHEON);
    deck.push(ids::HERO_OF_BLADEHOLD);
    deck.push(ids::PRECINCT_CAPTAIN);
    deck.push(ids::BLADE_SPLICER);

    // White spells
    deck.push(ids::SWORDS_TO_PLOWSHARES);
    deck.push(ids::PATH_TO_EXILE);
    deck.push(ids::WRATH_OF_GOD);
    deck.push(ids::DAY_OF_JUDGMENT);
    deck.push(ids::OBLIVION_RING);
    deck.push(ids::DISENCHANT);

    // White enchantments
    deck.push(ids::GLORIOUS_ANTHEM);
    deck.push(ids::HONOR_OF_THE_PURE);
    deck.push(ids::CRUSADE);

    // Colorless artifacts
    deck.push(ids::SOL_RING);
    deck.push(ids::SIGNAL_PEST);
    deck.push(ids::STEEL_OVERSEER);

    // Fill the rest with Plains to reach 100
    let nonland_count = deck.len();
    for _ in 0..(100 - nonland_count) {
        deck.push(ids::PLAINS);
    }

    assert_eq!(deck.len(), 100);
    (deck, commander)
}

/// Build a mono-green Commander deck with Thrun, the Last Troll as
/// the commander. Uses all available green cards plus colorless
/// artifacts, padded with Forests to reach 100 cards.
///
/// Returns (deck, commander_id).
pub fn thrun_commander_deck() -> (Vec<CardId>, CardId) {
    let commander = ids::THRUN_LAST_TROLL;
    let mut deck = Vec::new();

    // Commander
    deck.push(commander);

    // Green creatures
    deck.push(ids::LLANOWAR_ELVES);
    deck.push(ids::ELVISH_MYSTIC);
    deck.push(ids::GRIZZLY_BEARS);
    deck.push(ids::KALONIAN_TUSKER);
    deck.push(ids::LEATHERBACK_BALOTH);
    deck.push(ids::ELVISH_VISIONARY);
    deck.push(ids::TARMOGOYF);
    deck.push(ids::SCAVENGING_OOZE);
    deck.push(ids::STRANGLEROOT_GEIST);
    deck.push(ids::EXPERIMENT_ONE);
    deck.push(ids::DRYAD_MILITANT);
    deck.push(ids::RANCOR_BEAST);

    // Green spells
    deck.push(ids::GIANT_GROWTH);
    deck.push(ids::RANCOR);
    deck.push(ids::VINES_OF_VASTWOOD);
    deck.push(ids::COLLECTED_COMPANY);

    // Green enchantment
    deck.push(ids::GAEA_ANTHEM);

    // Colorless artifacts
    deck.push(ids::SOL_RING);
    deck.push(ids::SIGNAL_PEST);
    deck.push(ids::STEEL_OVERSEER);

    // Fill the rest with Forests to reach 100
    let nonland_count = deck.len();
    for _ in 0..(100 - nonland_count) {
        deck.push(ids::FOREST);
    }

    assert_eq!(deck.len(), 100);
    (deck, commander)
}

/// Build the Kinnan, Bonder Prodigy Commander deck (100-card singleton).
///
/// Returns (deck, commander_id, tutor_targets).
/// Tutor targets are the priority cards that tutors should search for,
/// ordered by strategic importance (Basalt Monolith first for infinite mana).
pub fn kinnan_commander_deck() -> (Vec<CardId>, CardId, Vec<CardId>) {
    let commander = ids::KINNAN_BONDER_PRODIGY;
    let mut deck = Vec::new();

    // Commander
    deck.push(commander);

    // Lands
    deck.push(ids::ANCIENT_TOMB);
    deck.push(ids::BOSEIJU_WHO_ENDURES);
    deck.push(ids::BREEDING_POOL);
    deck.push(ids::COMMAND_TOWER);
    deck.push(ids::FLOODED_STRAND);
    deck.push(ids::GAEAS_CRADLE);
    deck.push(ids::GEMSTONE_CAVERNS);
    deck.push(ids::INVENTORS_FAIR);
    deck.push(ids::ISLAND);
    deck.push(ids::MINAMO_SCHOOL);
    deck.push(ids::MISTY_RAINFOREST);
    deck.push(ids::MISTRISE_VILLAGE);
    deck.push(ids::OTAWARA_SOARING_CITY);
    deck.push(ids::SEAT_OF_THE_SYNOD);
    deck.push(ids::SHIFTING_WOODLAND);
    deck.push(ids::SNOW_COVERED_FOREST);
    deck.push(ids::SNOW_COVERED_ISLAND);
    deck.push(ids::TREASURE_VAULT);
    deck.push(ids::TREE_OF_TALES);
    deck.push(ids::TROPICAL_ISLAND);
    deck.push(ids::WATERLOGGED_GROVE);
    deck.push(ids::WINDSWEPT_HEATH);
    deck.push(ids::YAVIMAYA_COAST);
    deck.push(ids::FOREST);

    // Mana artifacts
    deck.push(ids::ARCANE_SIGNET);
    deck.push(ids::BASALT_MONOLITH);
    deck.push(ids::CHROME_MOX);
    deck.push(ids::FELLWAR_STONE);
    deck.push(ids::GRIM_MONOLITH);
    deck.push(ids::LOTUS_PETAL);
    deck.push(ids::MANA_VAULT);
    deck.push(ids::MOX_AMBER);
    deck.push(ids::MOX_DIAMOND);
    deck.push(ids::MOX_OPAL);
    deck.push(ids::MOONSILVER_KEY);
    deck.push(ids::SIMIC_SIGNET);
    deck.push(ids::SOL_RING);
    deck.push(ids::SPRINGLEAF_DRUM);
    deck.push(ids::TALISMAN_OF_CURIOSITY);

    // Other artifacts
    deck.push(ids::AGATHAS_SOUL_CAULDRON);
    deck.push(ids::THE_ONE_RING);
    deck.push(ids::MIRAGE_MIRROR);

    // Creatures
    deck.push(ids::BIRDS_OF_PARADISE);
    deck.push(ids::FYNDHORN_ELVES);
    deck.push(ids::LLANOWAR_ELVES);
    deck.push(ids::ELVISH_MYSTIC);
    deck.push(ids::DELIGHTED_HALFLING);
    deck.push(ids::BADGERMOLE_CUB);
    deck.push(ids::CLEVER_IMPERSONATOR);
    deck.push(ids::COLOSSAL_SKYTURTLE);
    deck.push(ids::CONSECRATED_SPHINX);
    deck.push(ids::DRIFT_OF_PHANTASMS);
    deck.push(ids::ELVISH_SPIRIT_GUIDE);
    deck.push(ids::ENDURANCE);
    deck.push(ids::ENDURING_VITALITY);
    deck.push(ids::FAERIE_MASTERMIND);
    deck.push(ids::FLESH_DUPLICATE);
    deck.push(ids::HIGH_FAE_TRICKSTER);
    deck.push(ids::HULLBREAKER_HORROR);
    deck.push(ids::MOCKINGBIRD);
    deck.push(ids::NEZAHAL_PRIMAL_TIDE);
    deck.push(ids::NYXBLOOM_ANCIENT);
    deck.push(ids::PHYREXIAN_METAMORPH);
    deck.push(ids::SEEDBORN_MUSE);
    deck.push(ids::THRASIOS_TRITON_HERO);
    deck.push(ids::TIDESPOUT_TYRANT);
    deck.push(ids::TROPHY_MAGE);
    deck.push(ids::WAN_SHI_TONG);
    deck.push(ids::WANDERING_ARCHAIC);

    // Instants
    deck.push(ids::AN_OFFER_YOU_CANT_REFUSE);
    deck.push(ids::CHORD_OF_CALLING);
    deck.push(ids::CROP_ROTATION);
    deck.push(ids::CYCLONIC_RIFT);
    deck.push(ids::FIERCE_GUARDIANSHIP);
    deck.push(ids::FLUSTERSTORM);
    deck.push(ids::FORCE_OF_NEGATION);
    deck.push(ids::FORCE_OF_WILL);
    deck.push(ids::INTO_THE_FLOOD_MAW);
    deck.push(ids::MENTAL_MISSTEP);
    deck.push(ids::MINDBREAK_TRAP);
    deck.push(ids::MYSTICAL_TUTOR);
    deck.push(ids::NOXIOUS_REVIVAL);
    deck.push(ids::PACT_OF_NEGATION);
    deck.push(ids::SWAN_SONG);
    deck.push(ids::VEIL_OF_SUMMER);
    deck.push(ids::WHIR_OF_INVENTION);
    deck.push(ids::WORLDLY_TUTOR);
    deck.push(ids::MUDDLE_THE_MIXTURE);

    // Sorceries
    deck.push(ids::FINALE_OF_DEVASTATION);
    deck.push(ids::GREEN_SUNS_ZENITH);
    deck.push(ids::NATURES_RHYTHM);

    // Enchantments
    deck.push(ids::MYSTIC_REMORA);
    deck.push(ids::RHYSTIC_STUDY);

    // Planeswalker
    deck.push(ids::TEZZERET_THE_SEEKER);

    // DFC / Battle cards
    deck.push(ids::BRIDGEWORKS_BATTLE);
    deck.push(ids::DISCIPLE_OF_FREYALISE);
    deck.push(ids::HYDROELECTRIC_SPECIMEN);
    deck.push(ids::INVASION_OF_IKORIA);
    deck.push(ids::SINK_INTO_STUPOR);

    assert_eq!(deck.len(), 100);

    // Priority tutor targets: combo pieces and high-impact cards.
    // Basalt Monolith is #1 — infinite mana with Kinnan's static ability.
    let tutor_targets = vec![
        ids::BASALT_MONOLITH,      // Infinite mana combo with Kinnan
        ids::GRIM_MONOLITH,        // Fast mana, doubles with Kinnan
        ids::THRASIOS_TRITON_HERO, // Mana sink to win with infinite mana
        ids::TIDESPOUT_TYRANT,     // Bounce engine, wins with infinite mana
        ids::HULLBREAKER_HORROR,   // Bounce engine, flash
        ids::CONSECRATED_SPHINX,   // Card advantage engine
        ids::NYXBLOOM_ANCIENT,     // Mana tripler
        ids::SEEDBORN_MUSE,        // Untap engine for Kinnan activations
        ids::SOL_RING,             // Best mana rock
        ids::TROPHY_MAGE,          // Tutors for Basalt Monolith (CMC 3)
        ids::THE_ONE_RING,         // Protection + card draw
    ];

    (deck, commander, tutor_targets)
}
