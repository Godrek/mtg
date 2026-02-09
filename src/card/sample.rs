//! Sample card definitions for testing — classic MTG cards.

use crate::card::*;
use crate::game::CardDatabase;
use crate::layers::{AffectedObjects, StaticAbility};
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

    // Phase 1A test cards
    pub const FIERY_CONCLUSION_ELEMENTAL: u64 = 200;
    pub const PYROCLASM_ELEMENTAL: u64 = 201;

    // =====================================================================
    // Phase 2A: Layered effects, anthems, lords, and expanded card pool
    // =====================================================================

    // --- White creatures ---
    pub const MOTHER_OF_RUNES: u64 = 300;
    pub const ELITE_VANGUARD: u64 = 301;
    pub const WHITE_KNIGHT: u64 = 302;
    pub const LEONIN_SKYHUNTER: u64 = 303;
    pub const BANESLAYER_ANGEL: u64 = 304;
    pub const THALIA_GUARDIAN: u64 = 305;
    pub const BRIMAZ_KING: u64 = 306;
    pub const SOLDIER_OF_THE_PANTHEON: u64 = 307;
    pub const HERO_OF_BLADEHOLD: u64 = 308;
    pub const PRECINCT_CAPTAIN: u64 = 309;

    // --- White spells ---
    pub const PATH_TO_EXILE: u64 = 310;
    pub const WRATH_OF_GOD: u64 = 311;
    pub const DAY_OF_JUDGMENT: u64 = 312;
    pub const OBLIVION_RING: u64 = 313;
    pub const DISENCHANT: u64 = 314;

    // --- White enchantments (anthem/layered effects) ---
    pub const GLORIOUS_ANTHEM: u64 = 315;
    pub const HONOR_OF_THE_PURE: u64 = 316;
    pub const CRUSADE: u64 = 317;
    pub const HUMILITY: u64 = 318;

    // --- Blue creatures ---
    pub const DELVER_OF_SECRETS: u64 = 320;
    pub const SNAPCASTER_MAGE: u64 = 321;
    pub const VENDILION_CLIQUE: u64 = 322;
    pub const MAN_O_WAR: u64 = 323;
    pub const SERENDIB_EFREET: u64 = 324;
    pub const PHANTASMAL_BEAR: u64 = 325;

    // --- Blue spells ---
    pub const MANA_LEAK: u64 = 326;
    pub const REMAND: u64 = 327;
    pub const BRAINSTORM: u64 = 328;
    pub const PONDER: u64 = 329;
    pub const UNSUMMON: u64 = 330;

    // --- Black creatures ---
    pub const DARK_CONFIDANT: u64 = 340;
    pub const HYPNOTIC_SPECTER: u64 = 341;
    pub const NANTUKO_SHADE: u64 = 342;
    pub const VAMPIRE_NIGHTHAWK: u64 = 343;
    pub const GATEKEEPER_OF_MALAKIR: u64 = 344;
    pub const BLOODGHAST: u64 = 345;
    pub const GERALF_MESSENGER: u64 = 346;
    pub const PHYREXIAN_OBLITERATOR: u64 = 347;
    pub const KNIGHT_OF_THE_EBON_LEGION: u64 = 348;

    // --- Black spells ---
    pub const DOOM_BLADE: u64 = 350;
    pub const GO_FOR_THE_THROAT: u64 = 351;
    pub const THOUGHTSEIZE: u64 = 352;
    pub const HYMN_TO_TOURACH: u64 = 353;
    pub const DIABOLIC_EDICT: u64 = 354;
    pub const TRAGIC_SLIP: u64 = 355;

    // --- Red creatures ---
    pub const ASH_ZEALOT: u64 = 360;
    pub const EMBER_HAULER: u64 = 361;
    pub const HELLRIDER: u64 = 362;
    pub const JACKAL_PUP: u64 = 363;
    pub const KELDON_MARAUDERS: u64 = 364;
    pub const VEXING_DEVIL: u64 = 365;
    pub const EIDOLON_OF_GREAT_REVEL: u64 = 366;
    pub const YOUNG_PYROMANCER: u64 = 367;
    pub const GOBLIN_CHAINWHIRLER: u64 = 368;

    // --- Red spells ---
    pub const CHAIN_LIGHTNING: u64 = 370;
    pub const SEARING_BLAZE: u64 = 371;
    pub const SKULLCRACK: u64 = 372;
    pub const FLAMES_OF_THE_BLOOD_HAND: u64 = 373;
    pub const SEARING_BLOOD: u64 = 374;

    // --- Green creatures ---
    pub const TARMOGOYF: u64 = 380;
    pub const SCAVENGING_OOZE: u64 = 381;
    pub const STRANGLEROOT_GEIST: u64 = 382;
    pub const WILD_NACATL: u64 = 383;
    pub const EXPERIMENT_ONE: u64 = 384;
    pub const DRYAD_MILITANT: u64 = 385;
    pub const THRUN_LAST_TROLL: u64 = 386;
    pub const RANCOR_BEAST: u64 = 387;

    // --- Green spells ---
    pub const RANCOR: u64 = 390;
    pub const VINES_OF_VASTWOOD: u64 = 391;
    pub const COLLECTED_COMPANY: u64 = 392;

    // --- Green enchantments (layered effects) ---
    pub const GAEA_ANTHEM: u64 = 393;

    // --- Artifacts ---
    pub const SOL_RING: u64 = 400;
    pub const SIGNAL_PEST: u64 = 401;
    pub const VAULT_SKIRGE: u64 = 402;
    pub const CRANIAL_PLATING: u64 = 403;
    pub const STEEL_OVERSEER: u64 = 404;

    // --- Multicolor ---
    pub const LIGHTNING_HELIX: u64 = 410;
    pub const TERMINATE: u64 = 411;
    pub const GEIST_OF_SAINT_TRAFT: u64 = 412;
    pub const FLEECEMANE_LION: u64 = 413;
    pub const TIDEHOLLOW_SCULLER: u64 = 414;
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
        activated_abilities: vec![], // simplified — no firebreathing
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
            description: "When Fiery Conclusion Elemental dies, it deals 2 damage to each player.".into(),
        }],
        starting_loyalty: None,
        static_abilities: vec![],
        enters_tapped: false,
        oracle_text: "When Fiery Conclusion Elemental dies, it deals 2 damage to each player.".into(),
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
            description: "When Pyroclasm Elemental dies, it deals 2 damage to each creature.".into(),
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
        keywords: vec![KeywordAbility::Flying, KeywordAbility::FirstStrike, KeywordAbility::Lifelink],
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
            description: "When Oblivion Ring enters the battlefield, exile another target nonland permanent.".into(),
        }],
        starting_loyalty: None,
        static_abilities: vec![],
        enters_tapped: false,
        oracle_text: "When Oblivion Ring enters the battlefield, exile another target nonland permanent.".into(),
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
        oracle_text: "All creatures lose all abilities and have base power and toughness 1/1.".into(),
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
        oracle_text: "When Phantasmal Bear becomes the target of a spell or ability, sacrifice it.".into(),
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

    // Brainstorm: U — Draw 3, put 2 back (simplified as draw 1)
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
        spell_effect: Some(Effect::DrawCards { count: 1 }),
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
        keywords: vec![KeywordAbility::Flying, KeywordAbility::Deathtouch, KeywordAbility::Lifelink],
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
        oracle_text: "{2}{B}: Knight of the Ebon Legion gets +3/+3 and gains deathtouch until end of turn.".into(),
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
        oracle_text: "{1}, Sacrifice Ember Hauler: Ember Hauler deals 2 damage to any target.".into(),
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
        oracle_text: "Whenever Jackal Pup is dealt damage, it deals that much damage to you.".into(),
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
        oracle_text: "Evolve. Remove two +1/+1 counters from Experiment One: Regenerate Experiment One.".into(),
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
        oracle_text: "This spell can't be countered. Hexproof. {1}{G}: Regenerate Thrun, the Last Troll.".into(),
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
        mana_abilities: vec![],
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
        oracle_text: "Battle cry. Signal Pest can't be blocked except by creatures with flying or reach.".into(),
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
            Effect::DealDamage { amount: 3, target: TargetSpec::CreatureOrPlayer },
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
        spell_effect: Some(Effect::AddMana { color: Some(Color::Black), amount: 3 }),
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
