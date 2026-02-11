//! Canonical card catalog and ID mapping.

use crate::card::CardId;
use crate::game::CardDatabase;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CardCatalogEntry {
    pub id: CardId,
    pub key: &'static str,
    pub name: &'static str,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CardImplementationStatus {
    pub id: CardId,
    pub key: &'static str,
    pub name: &'static str,
    pub effects_implemented: bool,
}

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
    pub const STOMPING_GROUND: u64 = 119;
    pub const ELVISH_VISIONARY: u64 = 120;
    pub const BLADE_SPLICER: u64 = 121;
    pub const SIEGE_GANG_COMMANDER: u64 = 122;
    pub const FIERY_CONCLUSION_ELEMENTAL: u64 = 200;
    pub const PYROCLASM_ELEMENTAL: u64 = 201;
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
    pub const PATH_TO_EXILE: u64 = 310;
    pub const WRATH_OF_GOD: u64 = 311;
    pub const DAY_OF_JUDGMENT: u64 = 312;
    pub const OBLIVION_RING: u64 = 313;
    pub const DISENCHANT: u64 = 314;
    pub const GLORIOUS_ANTHEM: u64 = 315;
    pub const HONOR_OF_THE_PURE: u64 = 316;
    pub const CRUSADE: u64 = 317;
    pub const HUMILITY: u64 = 318;
    pub const DELVER_OF_SECRETS: u64 = 320;
    pub const SNAPCASTER_MAGE: u64 = 321;
    pub const VENDILION_CLIQUE: u64 = 322;
    pub const MAN_O_WAR: u64 = 323;
    pub const SERENDIB_EFREET: u64 = 324;
    pub const PHANTASMAL_BEAR: u64 = 325;
    pub const MANA_LEAK: u64 = 326;
    pub const REMAND: u64 = 327;
    pub const BRAINSTORM: u64 = 328;
    pub const PONDER: u64 = 329;
    pub const UNSUMMON: u64 = 330;
    pub const DARK_CONFIDANT: u64 = 340;
    pub const HYPNOTIC_SPECTER: u64 = 341;
    pub const NANTUKO_SHADE: u64 = 342;
    pub const VAMPIRE_NIGHTHAWK: u64 = 343;
    pub const GATEKEEPER_OF_MALAKIR: u64 = 344;
    pub const BLOODGHAST: u64 = 345;
    pub const GERALF_MESSENGER: u64 = 346;
    pub const PHYREXIAN_OBLITERATOR: u64 = 347;
    pub const KNIGHT_OF_THE_EBON_LEGION: u64 = 348;
    pub const DOOM_BLADE: u64 = 350;
    pub const GO_FOR_THE_THROAT: u64 = 351;
    pub const THOUGHTSEIZE: u64 = 352;
    pub const HYMN_TO_TOURACH: u64 = 353;
    pub const DIABOLIC_EDICT: u64 = 354;
    pub const TRAGIC_SLIP: u64 = 355;
    pub const ASH_ZEALOT: u64 = 360;
    pub const EMBER_HAULER: u64 = 361;
    pub const HELLRIDER: u64 = 362;
    pub const JACKAL_PUP: u64 = 363;
    pub const KELDON_MARAUDERS: u64 = 364;
    pub const VEXING_DEVIL: u64 = 365;
    pub const EIDOLON_OF_GREAT_REVEL: u64 = 366;
    pub const YOUNG_PYROMANCER: u64 = 367;
    pub const GOBLIN_CHAINWHIRLER: u64 = 368;
    pub const CHAIN_LIGHTNING: u64 = 370;
    pub const SEARING_BLAZE: u64 = 371;
    pub const SKULLCRACK: u64 = 372;
    pub const FLAMES_OF_THE_BLOOD_HAND: u64 = 373;
    pub const SEARING_BLOOD: u64 = 374;
    pub const TARMOGOYF: u64 = 380;
    pub const SCAVENGING_OOZE: u64 = 381;
    pub const STRANGLEROOT_GEIST: u64 = 382;
    pub const WILD_NACATL: u64 = 383;
    pub const EXPERIMENT_ONE: u64 = 384;
    pub const DRYAD_MILITANT: u64 = 385;
    pub const THRUN_LAST_TROLL: u64 = 386;
    pub const RANCOR_BEAST: u64 = 387;
    pub const RANCOR: u64 = 390;
    pub const VINES_OF_VASTWOOD: u64 = 391;
    pub const COLLECTED_COMPANY: u64 = 392;
    pub const GAEA_ANTHEM: u64 = 393;
    pub const SOL_RING: u64 = 400;
    pub const SIGNAL_PEST: u64 = 401;
    pub const VAULT_SKIRGE: u64 = 402;
    pub const CRANIAL_PLATING: u64 = 403;
    pub const STEEL_OVERSEER: u64 = 404;
    pub const LIGHTNING_HELIX: u64 = 410;
    pub const TERMINATE: u64 = 411;
    pub const GEIST_OF_SAINT_TRAFT: u64 = 412;
    pub const FLEECEMANE_LION: u64 = 413;
    pub const TIDEHOLLOW_SCULLER: u64 = 414;
    pub const ANCIENT_TOMB: u64 = 500;
    pub const BOSEIJU_WHO_ENDURES: u64 = 501;
    pub const BREEDING_POOL: u64 = 502;
    pub const COMMAND_TOWER: u64 = 503;
    pub const FLOODED_STRAND: u64 = 504;
    pub const GAEAS_CRADLE: u64 = 505;
    pub const GEMSTONE_CAVERNS: u64 = 506;
    pub const INVENTORS_FAIR: u64 = 507;
    pub const MINAMO_SCHOOL: u64 = 508;
    pub const MISTY_RAINFOREST: u64 = 509;
    pub const MISTRISE_VILLAGE: u64 = 510;
    pub const OTAWARA_SOARING_CITY: u64 = 511;
    pub const SEAT_OF_THE_SYNOD: u64 = 512;
    pub const SHIFTING_WOODLAND: u64 = 513;
    pub const SNOW_COVERED_FOREST: u64 = 514;
    pub const SNOW_COVERED_ISLAND: u64 = 515;
    pub const TREASURE_VAULT: u64 = 516;
    pub const TREE_OF_TALES: u64 = 517;
    pub const TROPICAL_ISLAND: u64 = 518;
    pub const WATERLOGGED_GROVE: u64 = 519;
    pub const WINDSWEPT_HEATH: u64 = 520;
    pub const YAVIMAYA_COAST: u64 = 521;
    pub const ARCANE_SIGNET: u64 = 550;
    pub const BASALT_MONOLITH: u64 = 551;
    pub const CHROME_MOX: u64 = 552;
    pub const FELLWAR_STONE: u64 = 553;
    pub const GRIM_MONOLITH: u64 = 554;
    pub const LOTUS_PETAL: u64 = 555;
    pub const MANA_VAULT: u64 = 556;
    pub const MOX_AMBER: u64 = 557;
    pub const MOX_DIAMOND: u64 = 558;
    pub const MOX_OPAL: u64 = 559;
    pub const MOONSILVER_KEY: u64 = 560;
    pub const SIMIC_SIGNET: u64 = 561;
    pub const SPRINGLEAF_DRUM: u64 = 562;
    pub const TALISMAN_OF_CURIOSITY: u64 = 563;
    pub const AGATHAS_SOUL_CAULDRON: u64 = 580;
    pub const THE_ONE_RING: u64 = 581;
    pub const MIRAGE_MIRROR: u64 = 582;
    pub const KINNAN_BONDER_PRODIGY: u64 = 600;
    pub const BIRDS_OF_PARADISE: u64 = 601;
    pub const FYNDHORN_ELVES: u64 = 602;
    pub const DELIGHTED_HALFLING: u64 = 603;
    pub const BADGERMOLE_CUB: u64 = 604;
    pub const CLEVER_IMPERSONATOR: u64 = 605;
    pub const COLOSSAL_SKYTURTLE: u64 = 606;
    pub const CONSECRATED_SPHINX: u64 = 607;
    pub const DRIFT_OF_PHANTASMS: u64 = 608;
    pub const ELVISH_SPIRIT_GUIDE: u64 = 609;
    pub const ENDURANCE: u64 = 610;
    pub const ENDURING_VITALITY: u64 = 611;
    pub const FAERIE_MASTERMIND: u64 = 612;
    pub const FLESH_DUPLICATE: u64 = 613;
    pub const HIGH_FAE_TRICKSTER: u64 = 614;
    pub const HULLBREAKER_HORROR: u64 = 615;
    pub const MOCKINGBIRD: u64 = 616;
    pub const NEZAHAL_PRIMAL_TIDE: u64 = 617;
    pub const NYXBLOOM_ANCIENT: u64 = 618;
    pub const PHYREXIAN_METAMORPH: u64 = 619;
    pub const SEEDBORN_MUSE: u64 = 620;
    pub const THRASIOS_TRITON_HERO: u64 = 621;
    pub const TIDESPOUT_TYRANT: u64 = 622;
    pub const TROPHY_MAGE: u64 = 623;
    pub const WAN_SHI_TONG: u64 = 624;
    pub const WANDERING_ARCHAIC: u64 = 625;
    pub const AN_OFFER_YOU_CANT_REFUSE: u64 = 700;
    pub const CHORD_OF_CALLING: u64 = 701;
    pub const CROP_ROTATION: u64 = 702;
    pub const CYCLONIC_RIFT: u64 = 703;
    pub const FIERCE_GUARDIANSHIP: u64 = 704;
    pub const FLUSTERSTORM: u64 = 705;
    pub const FORCE_OF_NEGATION: u64 = 706;
    pub const FORCE_OF_WILL: u64 = 707;
    pub const INTO_THE_FLOOD_MAW: u64 = 708;
    pub const MENTAL_MISSTEP: u64 = 709;
    pub const MINDBREAK_TRAP: u64 = 710;
    pub const MYSTICAL_TUTOR: u64 = 711;
    pub const NOXIOUS_REVIVAL: u64 = 712;
    pub const PACT_OF_NEGATION: u64 = 713;
    pub const SWAN_SONG: u64 = 714;
    pub const VEIL_OF_SUMMER: u64 = 715;
    pub const WHIR_OF_INVENTION: u64 = 716;
    pub const WORLDLY_TUTOR: u64 = 717;
    pub const MUDDLE_THE_MIXTURE: u64 = 718;
    pub const FINALE_OF_DEVASTATION: u64 = 750;
    pub const GREEN_SUNS_ZENITH: u64 = 751;
    pub const NATURES_RHYTHM: u64 = 752;
    pub const MYSTIC_REMORA: u64 = 770;
    pub const RHYSTIC_STUDY: u64 = 771;
    pub const TEZZERET_THE_SEEKER: u64 = 790;
    pub const BRIDGEWORKS_BATTLE: u64 = 800;
    pub const DISCIPLE_OF_FREYALISE: u64 = 801;
    pub const HYDROELECTRIC_SPECIMEN: u64 = 802;
    pub const INVASION_OF_IKORIA: u64 = 803;
    pub const SINK_INTO_STUPOR: u64 = 804;
}

pub const ALL_CARDS: &[CardCatalogEntry] = &[
    CardCatalogEntry {
        id: ids::MOUNTAIN,
        key: "MOUNTAIN",
        name: "Mountain",
    },
    CardCatalogEntry {
        id: ids::FOREST,
        key: "FOREST",
        name: "Forest",
    },
    CardCatalogEntry {
        id: ids::PLAINS,
        key: "PLAINS",
        name: "Plains",
    },
    CardCatalogEntry {
        id: ids::ISLAND,
        key: "ISLAND",
        name: "Island",
    },
    CardCatalogEntry {
        id: ids::SWAMP,
        key: "SWAMP",
        name: "Swamp",
    },
    CardCatalogEntry {
        id: ids::LIGHTNING_BOLT,
        key: "LIGHTNING_BOLT",
        name: "Lightning Bolt",
    },
    CardCatalogEntry {
        id: ids::GIANT_GROWTH,
        key: "GIANT_GROWTH",
        name: "Giant Growth",
    },
    CardCatalogEntry {
        id: ids::GRIZZLY_BEARS,
        key: "GRIZZLY_BEARS",
        name: "Grizzly Bears",
    },
    CardCatalogEntry {
        id: ids::GREY_OGRE,
        key: "GREY_OGRE",
        name: "Grey Ogre",
    },
    CardCatalogEntry {
        id: ids::SERRA_ANGEL,
        key: "SERRA_ANGEL",
        name: "Serra Angel",
    },
    CardCatalogEntry {
        id: ids::SHIVAN_DRAGON,
        key: "SHIVAN_DRAGON",
        name: "Shivan Dragon",
    },
    CardCatalogEntry {
        id: ids::LLANOWAR_ELVES,
        key: "LLANOWAR_ELVES",
        name: "Llanowar Elves",
    },
    CardCatalogEntry {
        id: ids::SWORDS_TO_PLOWSHARES,
        key: "SWORDS_TO_PLOWSHARES",
        name: "Swords to Plowshares",
    },
    CardCatalogEntry {
        id: ids::COUNTERSPELL,
        key: "COUNTERSPELL",
        name: "Counterspell",
    },
    CardCatalogEntry {
        id: ids::DARK_RITUAL,
        key: "DARK_RITUAL",
        name: "Dark Ritual",
    },
    CardCatalogEntry {
        id: ids::SAVANNAH_LIONS,
        key: "SAVANNAH_LIONS",
        name: "Savannah Lions",
    },
    CardCatalogEntry {
        id: ids::SHOCK,
        key: "SHOCK",
        name: "Shock",
    },
    CardCatalogEntry {
        id: ids::GOBLIN_GUIDE,
        key: "GOBLIN_GUIDE",
        name: "Goblin Guide",
    },
    CardCatalogEntry {
        id: ids::ELVISH_MYSTIC,
        key: "ELVISH_MYSTIC",
        name: "Elvish Mystic",
    },
    CardCatalogEntry {
        id: ids::KALONIAN_TUSKER,
        key: "KALONIAN_TUSKER",
        name: "Kalonian Tusker",
    },
    CardCatalogEntry {
        id: ids::LEATHERBACK_BALOTH,
        key: "LEATHERBACK_BALOTH",
        name: "Leatherback Baloth",
    },
    CardCatalogEntry {
        id: ids::MONASTERY_SWIFTSPEAR,
        key: "MONASTERY_SWIFTSPEAR",
        name: "Monastery Swiftspear",
    },
    CardCatalogEntry {
        id: ids::LAVA_SPIKE,
        key: "LAVA_SPIKE",
        name: "Lava Spike",
    },
    CardCatalogEntry {
        id: ids::RIFT_BOLT,
        key: "RIFT_BOLT",
        name: "Rift Bolt",
    },
    CardCatalogEntry {
        id: ids::STOMPING_GROUND,
        key: "STOMPING_GROUND",
        name: "Stomping Ground",
    },
    CardCatalogEntry {
        id: ids::ELVISH_VISIONARY,
        key: "ELVISH_VISIONARY",
        name: "Elvish Visionary",
    },
    CardCatalogEntry {
        id: ids::BLADE_SPLICER,
        key: "BLADE_SPLICER",
        name: "Blade Splicer",
    },
    CardCatalogEntry {
        id: ids::SIEGE_GANG_COMMANDER,
        key: "SIEGE_GANG_COMMANDER",
        name: "Siege Gang Commander",
    },
    CardCatalogEntry {
        id: ids::FIERY_CONCLUSION_ELEMENTAL,
        key: "FIERY_CONCLUSION_ELEMENTAL",
        name: "Fiery Conclusion Elemental",
    },
    CardCatalogEntry {
        id: ids::PYROCLASM_ELEMENTAL,
        key: "PYROCLASM_ELEMENTAL",
        name: "Pyroclasm Elemental",
    },
    CardCatalogEntry {
        id: ids::MOTHER_OF_RUNES,
        key: "MOTHER_OF_RUNES",
        name: "Mother of Runes",
    },
    CardCatalogEntry {
        id: ids::ELITE_VANGUARD,
        key: "ELITE_VANGUARD",
        name: "Elite Vanguard",
    },
    CardCatalogEntry {
        id: ids::WHITE_KNIGHT,
        key: "WHITE_KNIGHT",
        name: "White Knight",
    },
    CardCatalogEntry {
        id: ids::LEONIN_SKYHUNTER,
        key: "LEONIN_SKYHUNTER",
        name: "Leonin Skyhunter",
    },
    CardCatalogEntry {
        id: ids::BANESLAYER_ANGEL,
        key: "BANESLAYER_ANGEL",
        name: "Baneslayer Angel",
    },
    CardCatalogEntry {
        id: ids::THALIA_GUARDIAN,
        key: "THALIA_GUARDIAN",
        name: "Thalia Guardian",
    },
    CardCatalogEntry {
        id: ids::BRIMAZ_KING,
        key: "BRIMAZ_KING",
        name: "Brimaz King",
    },
    CardCatalogEntry {
        id: ids::SOLDIER_OF_THE_PANTHEON,
        key: "SOLDIER_OF_THE_PANTHEON",
        name: "Soldier of the Pantheon",
    },
    CardCatalogEntry {
        id: ids::HERO_OF_BLADEHOLD,
        key: "HERO_OF_BLADEHOLD",
        name: "Hero of Bladehold",
    },
    CardCatalogEntry {
        id: ids::PRECINCT_CAPTAIN,
        key: "PRECINCT_CAPTAIN",
        name: "Precinct Captain",
    },
    CardCatalogEntry {
        id: ids::PATH_TO_EXILE,
        key: "PATH_TO_EXILE",
        name: "Path to Exile",
    },
    CardCatalogEntry {
        id: ids::WRATH_OF_GOD,
        key: "WRATH_OF_GOD",
        name: "Wrath of God",
    },
    CardCatalogEntry {
        id: ids::DAY_OF_JUDGMENT,
        key: "DAY_OF_JUDGMENT",
        name: "Day of Judgment",
    },
    CardCatalogEntry {
        id: ids::OBLIVION_RING,
        key: "OBLIVION_RING",
        name: "Oblivion Ring",
    },
    CardCatalogEntry {
        id: ids::DISENCHANT,
        key: "DISENCHANT",
        name: "Disenchant",
    },
    CardCatalogEntry {
        id: ids::GLORIOUS_ANTHEM,
        key: "GLORIOUS_ANTHEM",
        name: "Glorious Anthem",
    },
    CardCatalogEntry {
        id: ids::HONOR_OF_THE_PURE,
        key: "HONOR_OF_THE_PURE",
        name: "Honor of the Pure",
    },
    CardCatalogEntry {
        id: ids::CRUSADE,
        key: "CRUSADE",
        name: "Crusade",
    },
    CardCatalogEntry {
        id: ids::HUMILITY,
        key: "HUMILITY",
        name: "Humility",
    },
    CardCatalogEntry {
        id: ids::DELVER_OF_SECRETS,
        key: "DELVER_OF_SECRETS",
        name: "Delver of Secrets",
    },
    CardCatalogEntry {
        id: ids::SNAPCASTER_MAGE,
        key: "SNAPCASTER_MAGE",
        name: "Snapcaster Mage",
    },
    CardCatalogEntry {
        id: ids::VENDILION_CLIQUE,
        key: "VENDILION_CLIQUE",
        name: "Vendilion Clique",
    },
    CardCatalogEntry {
        id: ids::MAN_O_WAR,
        key: "MAN_O_WAR",
        name: "Man O War",
    },
    CardCatalogEntry {
        id: ids::SERENDIB_EFREET,
        key: "SERENDIB_EFREET",
        name: "Serendib Efreet",
    },
    CardCatalogEntry {
        id: ids::PHANTASMAL_BEAR,
        key: "PHANTASMAL_BEAR",
        name: "Phantasmal Bear",
    },
    CardCatalogEntry {
        id: ids::MANA_LEAK,
        key: "MANA_LEAK",
        name: "Mana Leak",
    },
    CardCatalogEntry {
        id: ids::REMAND,
        key: "REMAND",
        name: "Remand",
    },
    CardCatalogEntry {
        id: ids::BRAINSTORM,
        key: "BRAINSTORM",
        name: "Brainstorm",
    },
    CardCatalogEntry {
        id: ids::PONDER,
        key: "PONDER",
        name: "Ponder",
    },
    CardCatalogEntry {
        id: ids::UNSUMMON,
        key: "UNSUMMON",
        name: "Unsummon",
    },
    CardCatalogEntry {
        id: ids::DARK_CONFIDANT,
        key: "DARK_CONFIDANT",
        name: "Dark Confidant",
    },
    CardCatalogEntry {
        id: ids::HYPNOTIC_SPECTER,
        key: "HYPNOTIC_SPECTER",
        name: "Hypnotic Specter",
    },
    CardCatalogEntry {
        id: ids::NANTUKO_SHADE,
        key: "NANTUKO_SHADE",
        name: "Nantuko Shade",
    },
    CardCatalogEntry {
        id: ids::VAMPIRE_NIGHTHAWK,
        key: "VAMPIRE_NIGHTHAWK",
        name: "Vampire Nighthawk",
    },
    CardCatalogEntry {
        id: ids::GATEKEEPER_OF_MALAKIR,
        key: "GATEKEEPER_OF_MALAKIR",
        name: "Gatekeeper of Malakir",
    },
    CardCatalogEntry {
        id: ids::BLOODGHAST,
        key: "BLOODGHAST",
        name: "Bloodghast",
    },
    CardCatalogEntry {
        id: ids::GERALF_MESSENGER,
        key: "GERALF_MESSENGER",
        name: "Geralf Messenger",
    },
    CardCatalogEntry {
        id: ids::PHYREXIAN_OBLITERATOR,
        key: "PHYREXIAN_OBLITERATOR",
        name: "Phyrexian Obliterator",
    },
    CardCatalogEntry {
        id: ids::KNIGHT_OF_THE_EBON_LEGION,
        key: "KNIGHT_OF_THE_EBON_LEGION",
        name: "Knight of the Ebon Legion",
    },
    CardCatalogEntry {
        id: ids::DOOM_BLADE,
        key: "DOOM_BLADE",
        name: "Doom Blade",
    },
    CardCatalogEntry {
        id: ids::GO_FOR_THE_THROAT,
        key: "GO_FOR_THE_THROAT",
        name: "Go For the Throat",
    },
    CardCatalogEntry {
        id: ids::THOUGHTSEIZE,
        key: "THOUGHTSEIZE",
        name: "Thoughtseize",
    },
    CardCatalogEntry {
        id: ids::HYMN_TO_TOURACH,
        key: "HYMN_TO_TOURACH",
        name: "Hymn to Tourach",
    },
    CardCatalogEntry {
        id: ids::DIABOLIC_EDICT,
        key: "DIABOLIC_EDICT",
        name: "Diabolic Edict",
    },
    CardCatalogEntry {
        id: ids::TRAGIC_SLIP,
        key: "TRAGIC_SLIP",
        name: "Tragic Slip",
    },
    CardCatalogEntry {
        id: ids::ASH_ZEALOT,
        key: "ASH_ZEALOT",
        name: "Ash Zealot",
    },
    CardCatalogEntry {
        id: ids::EMBER_HAULER,
        key: "EMBER_HAULER",
        name: "Ember Hauler",
    },
    CardCatalogEntry {
        id: ids::HELLRIDER,
        key: "HELLRIDER",
        name: "Hellrider",
    },
    CardCatalogEntry {
        id: ids::JACKAL_PUP,
        key: "JACKAL_PUP",
        name: "Jackal Pup",
    },
    CardCatalogEntry {
        id: ids::KELDON_MARAUDERS,
        key: "KELDON_MARAUDERS",
        name: "Keldon Marauders",
    },
    CardCatalogEntry {
        id: ids::VEXING_DEVIL,
        key: "VEXING_DEVIL",
        name: "Vexing Devil",
    },
    CardCatalogEntry {
        id: ids::EIDOLON_OF_GREAT_REVEL,
        key: "EIDOLON_OF_GREAT_REVEL",
        name: "Eidolon of Great Revel",
    },
    CardCatalogEntry {
        id: ids::YOUNG_PYROMANCER,
        key: "YOUNG_PYROMANCER",
        name: "Young Pyromancer",
    },
    CardCatalogEntry {
        id: ids::GOBLIN_CHAINWHIRLER,
        key: "GOBLIN_CHAINWHIRLER",
        name: "Goblin Chainwhirler",
    },
    CardCatalogEntry {
        id: ids::CHAIN_LIGHTNING,
        key: "CHAIN_LIGHTNING",
        name: "Chain Lightning",
    },
    CardCatalogEntry {
        id: ids::SEARING_BLAZE,
        key: "SEARING_BLAZE",
        name: "Searing Blaze",
    },
    CardCatalogEntry {
        id: ids::SKULLCRACK,
        key: "SKULLCRACK",
        name: "Skullcrack",
    },
    CardCatalogEntry {
        id: ids::FLAMES_OF_THE_BLOOD_HAND,
        key: "FLAMES_OF_THE_BLOOD_HAND",
        name: "Flames of the Blood Hand",
    },
    CardCatalogEntry {
        id: ids::SEARING_BLOOD,
        key: "SEARING_BLOOD",
        name: "Searing Blood",
    },
    CardCatalogEntry {
        id: ids::TARMOGOYF,
        key: "TARMOGOYF",
        name: "Tarmogoyf",
    },
    CardCatalogEntry {
        id: ids::SCAVENGING_OOZE,
        key: "SCAVENGING_OOZE",
        name: "Scavenging Ooze",
    },
    CardCatalogEntry {
        id: ids::STRANGLEROOT_GEIST,
        key: "STRANGLEROOT_GEIST",
        name: "Strangleroot Geist",
    },
    CardCatalogEntry {
        id: ids::WILD_NACATL,
        key: "WILD_NACATL",
        name: "Wild Nacatl",
    },
    CardCatalogEntry {
        id: ids::EXPERIMENT_ONE,
        key: "EXPERIMENT_ONE",
        name: "Experiment One",
    },
    CardCatalogEntry {
        id: ids::DRYAD_MILITANT,
        key: "DRYAD_MILITANT",
        name: "Dryad Militant",
    },
    CardCatalogEntry {
        id: ids::THRUN_LAST_TROLL,
        key: "THRUN_LAST_TROLL",
        name: "Thrun Last Troll",
    },
    CardCatalogEntry {
        id: ids::RANCOR_BEAST,
        key: "RANCOR_BEAST",
        name: "Rancor Beast",
    },
    CardCatalogEntry {
        id: ids::RANCOR,
        key: "RANCOR",
        name: "Rancor",
    },
    CardCatalogEntry {
        id: ids::VINES_OF_VASTWOOD,
        key: "VINES_OF_VASTWOOD",
        name: "Vines of Vastwood",
    },
    CardCatalogEntry {
        id: ids::COLLECTED_COMPANY,
        key: "COLLECTED_COMPANY",
        name: "Collected Company",
    },
    CardCatalogEntry {
        id: ids::GAEA_ANTHEM,
        key: "GAEA_ANTHEM",
        name: "Gaea Anthem",
    },
    CardCatalogEntry {
        id: ids::SOL_RING,
        key: "SOL_RING",
        name: "Sol Ring",
    },
    CardCatalogEntry {
        id: ids::SIGNAL_PEST,
        key: "SIGNAL_PEST",
        name: "Signal Pest",
    },
    CardCatalogEntry {
        id: ids::VAULT_SKIRGE,
        key: "VAULT_SKIRGE",
        name: "Vault Skirge",
    },
    CardCatalogEntry {
        id: ids::CRANIAL_PLATING,
        key: "CRANIAL_PLATING",
        name: "Cranial Plating",
    },
    CardCatalogEntry {
        id: ids::STEEL_OVERSEER,
        key: "STEEL_OVERSEER",
        name: "Steel Overseer",
    },
    CardCatalogEntry {
        id: ids::LIGHTNING_HELIX,
        key: "LIGHTNING_HELIX",
        name: "Lightning Helix",
    },
    CardCatalogEntry {
        id: ids::TERMINATE,
        key: "TERMINATE",
        name: "Terminate",
    },
    CardCatalogEntry {
        id: ids::GEIST_OF_SAINT_TRAFT,
        key: "GEIST_OF_SAINT_TRAFT",
        name: "Geist of Saint Traft",
    },
    CardCatalogEntry {
        id: ids::FLEECEMANE_LION,
        key: "FLEECEMANE_LION",
        name: "Fleecemane Lion",
    },
    CardCatalogEntry {
        id: ids::TIDEHOLLOW_SCULLER,
        key: "TIDEHOLLOW_SCULLER",
        name: "Tidehollow Sculler",
    },
    CardCatalogEntry {
        id: ids::ANCIENT_TOMB,
        key: "ANCIENT_TOMB",
        name: "Ancient Tomb",
    },
    CardCatalogEntry {
        id: ids::BOSEIJU_WHO_ENDURES,
        key: "BOSEIJU_WHO_ENDURES",
        name: "Boseiju Who Endures",
    },
    CardCatalogEntry {
        id: ids::BREEDING_POOL,
        key: "BREEDING_POOL",
        name: "Breeding Pool",
    },
    CardCatalogEntry {
        id: ids::COMMAND_TOWER,
        key: "COMMAND_TOWER",
        name: "Command Tower",
    },
    CardCatalogEntry {
        id: ids::FLOODED_STRAND,
        key: "FLOODED_STRAND",
        name: "Flooded Strand",
    },
    CardCatalogEntry {
        id: ids::GAEAS_CRADLE,
        key: "GAEAS_CRADLE",
        name: "Gaeas Cradle",
    },
    CardCatalogEntry {
        id: ids::GEMSTONE_CAVERNS,
        key: "GEMSTONE_CAVERNS",
        name: "Gemstone Caverns",
    },
    CardCatalogEntry {
        id: ids::INVENTORS_FAIR,
        key: "INVENTORS_FAIR",
        name: "Inventors Fair",
    },
    CardCatalogEntry {
        id: ids::MINAMO_SCHOOL,
        key: "MINAMO_SCHOOL",
        name: "Minamo School",
    },
    CardCatalogEntry {
        id: ids::MISTY_RAINFOREST,
        key: "MISTY_RAINFOREST",
        name: "Misty Rainforest",
    },
    CardCatalogEntry {
        id: ids::MISTRISE_VILLAGE,
        key: "MISTRISE_VILLAGE",
        name: "Mistrise Village",
    },
    CardCatalogEntry {
        id: ids::OTAWARA_SOARING_CITY,
        key: "OTAWARA_SOARING_CITY",
        name: "Otawara Soaring City",
    },
    CardCatalogEntry {
        id: ids::SEAT_OF_THE_SYNOD,
        key: "SEAT_OF_THE_SYNOD",
        name: "Seat of the Synod",
    },
    CardCatalogEntry {
        id: ids::SHIFTING_WOODLAND,
        key: "SHIFTING_WOODLAND",
        name: "Shifting Woodland",
    },
    CardCatalogEntry {
        id: ids::SNOW_COVERED_FOREST,
        key: "SNOW_COVERED_FOREST",
        name: "Snow Covered Forest",
    },
    CardCatalogEntry {
        id: ids::SNOW_COVERED_ISLAND,
        key: "SNOW_COVERED_ISLAND",
        name: "Snow Covered Island",
    },
    CardCatalogEntry {
        id: ids::TREASURE_VAULT,
        key: "TREASURE_VAULT",
        name: "Treasure Vault",
    },
    CardCatalogEntry {
        id: ids::TREE_OF_TALES,
        key: "TREE_OF_TALES",
        name: "Tree of Tales",
    },
    CardCatalogEntry {
        id: ids::TROPICAL_ISLAND,
        key: "TROPICAL_ISLAND",
        name: "Tropical Island",
    },
    CardCatalogEntry {
        id: ids::WATERLOGGED_GROVE,
        key: "WATERLOGGED_GROVE",
        name: "Waterlogged Grove",
    },
    CardCatalogEntry {
        id: ids::WINDSWEPT_HEATH,
        key: "WINDSWEPT_HEATH",
        name: "Windswept Heath",
    },
    CardCatalogEntry {
        id: ids::YAVIMAYA_COAST,
        key: "YAVIMAYA_COAST",
        name: "Yavimaya Coast",
    },
    CardCatalogEntry {
        id: ids::ARCANE_SIGNET,
        key: "ARCANE_SIGNET",
        name: "Arcane Signet",
    },
    CardCatalogEntry {
        id: ids::BASALT_MONOLITH,
        key: "BASALT_MONOLITH",
        name: "Basalt Monolith",
    },
    CardCatalogEntry {
        id: ids::CHROME_MOX,
        key: "CHROME_MOX",
        name: "Chrome Mox",
    },
    CardCatalogEntry {
        id: ids::FELLWAR_STONE,
        key: "FELLWAR_STONE",
        name: "Fellwar Stone",
    },
    CardCatalogEntry {
        id: ids::GRIM_MONOLITH,
        key: "GRIM_MONOLITH",
        name: "Grim Monolith",
    },
    CardCatalogEntry {
        id: ids::LOTUS_PETAL,
        key: "LOTUS_PETAL",
        name: "Lotus Petal",
    },
    CardCatalogEntry {
        id: ids::MANA_VAULT,
        key: "MANA_VAULT",
        name: "Mana Vault",
    },
    CardCatalogEntry {
        id: ids::MOX_AMBER,
        key: "MOX_AMBER",
        name: "Mox Amber",
    },
    CardCatalogEntry {
        id: ids::MOX_DIAMOND,
        key: "MOX_DIAMOND",
        name: "Mox Diamond",
    },
    CardCatalogEntry {
        id: ids::MOX_OPAL,
        key: "MOX_OPAL",
        name: "Mox Opal",
    },
    CardCatalogEntry {
        id: ids::MOONSILVER_KEY,
        key: "MOONSILVER_KEY",
        name: "Moonsilver Key",
    },
    CardCatalogEntry {
        id: ids::SIMIC_SIGNET,
        key: "SIMIC_SIGNET",
        name: "Simic Signet",
    },
    CardCatalogEntry {
        id: ids::SPRINGLEAF_DRUM,
        key: "SPRINGLEAF_DRUM",
        name: "Springleaf Drum",
    },
    CardCatalogEntry {
        id: ids::TALISMAN_OF_CURIOSITY,
        key: "TALISMAN_OF_CURIOSITY",
        name: "Talisman of Curiosity",
    },
    CardCatalogEntry {
        id: ids::AGATHAS_SOUL_CAULDRON,
        key: "AGATHAS_SOUL_CAULDRON",
        name: "Agathas Soul Cauldron",
    },
    CardCatalogEntry {
        id: ids::THE_ONE_RING,
        key: "THE_ONE_RING",
        name: "the One Ring",
    },
    CardCatalogEntry {
        id: ids::MIRAGE_MIRROR,
        key: "MIRAGE_MIRROR",
        name: "Mirage Mirror",
    },
    CardCatalogEntry {
        id: ids::KINNAN_BONDER_PRODIGY,
        key: "KINNAN_BONDER_PRODIGY",
        name: "Kinnan Bonder Prodigy",
    },
    CardCatalogEntry {
        id: ids::BIRDS_OF_PARADISE,
        key: "BIRDS_OF_PARADISE",
        name: "Birds of Paradise",
    },
    CardCatalogEntry {
        id: ids::FYNDHORN_ELVES,
        key: "FYNDHORN_ELVES",
        name: "Fyndhorn Elves",
    },
    CardCatalogEntry {
        id: ids::DELIGHTED_HALFLING,
        key: "DELIGHTED_HALFLING",
        name: "Delighted Halfling",
    },
    CardCatalogEntry {
        id: ids::BADGERMOLE_CUB,
        key: "BADGERMOLE_CUB",
        name: "Badgermole Cub",
    },
    CardCatalogEntry {
        id: ids::CLEVER_IMPERSONATOR,
        key: "CLEVER_IMPERSONATOR",
        name: "Clever Impersonator",
    },
    CardCatalogEntry {
        id: ids::COLOSSAL_SKYTURTLE,
        key: "COLOSSAL_SKYTURTLE",
        name: "Colossal Skyturtle",
    },
    CardCatalogEntry {
        id: ids::CONSECRATED_SPHINX,
        key: "CONSECRATED_SPHINX",
        name: "Consecrated Sphinx",
    },
    CardCatalogEntry {
        id: ids::DRIFT_OF_PHANTASMS,
        key: "DRIFT_OF_PHANTASMS",
        name: "Drift of Phantasms",
    },
    CardCatalogEntry {
        id: ids::ELVISH_SPIRIT_GUIDE,
        key: "ELVISH_SPIRIT_GUIDE",
        name: "Elvish Spirit Guide",
    },
    CardCatalogEntry {
        id: ids::ENDURANCE,
        key: "ENDURANCE",
        name: "Endurance",
    },
    CardCatalogEntry {
        id: ids::ENDURING_VITALITY,
        key: "ENDURING_VITALITY",
        name: "Enduring Vitality",
    },
    CardCatalogEntry {
        id: ids::FAERIE_MASTERMIND,
        key: "FAERIE_MASTERMIND",
        name: "Faerie Mastermind",
    },
    CardCatalogEntry {
        id: ids::FLESH_DUPLICATE,
        key: "FLESH_DUPLICATE",
        name: "Flesh Duplicate",
    },
    CardCatalogEntry {
        id: ids::HIGH_FAE_TRICKSTER,
        key: "HIGH_FAE_TRICKSTER",
        name: "High Fae Trickster",
    },
    CardCatalogEntry {
        id: ids::HULLBREAKER_HORROR,
        key: "HULLBREAKER_HORROR",
        name: "Hullbreaker Horror",
    },
    CardCatalogEntry {
        id: ids::MOCKINGBIRD,
        key: "MOCKINGBIRD",
        name: "Mockingbird",
    },
    CardCatalogEntry {
        id: ids::NEZAHAL_PRIMAL_TIDE,
        key: "NEZAHAL_PRIMAL_TIDE",
        name: "Nezahal Primal Tide",
    },
    CardCatalogEntry {
        id: ids::NYXBLOOM_ANCIENT,
        key: "NYXBLOOM_ANCIENT",
        name: "Nyxbloom Ancient",
    },
    CardCatalogEntry {
        id: ids::PHYREXIAN_METAMORPH,
        key: "PHYREXIAN_METAMORPH",
        name: "Phyrexian Metamorph",
    },
    CardCatalogEntry {
        id: ids::SEEDBORN_MUSE,
        key: "SEEDBORN_MUSE",
        name: "Seedborn Muse",
    },
    CardCatalogEntry {
        id: ids::THRASIOS_TRITON_HERO,
        key: "THRASIOS_TRITON_HERO",
        name: "Thrasios Triton Hero",
    },
    CardCatalogEntry {
        id: ids::TIDESPOUT_TYRANT,
        key: "TIDESPOUT_TYRANT",
        name: "Tidespout Tyrant",
    },
    CardCatalogEntry {
        id: ids::TROPHY_MAGE,
        key: "TROPHY_MAGE",
        name: "Trophy Mage",
    },
    CardCatalogEntry {
        id: ids::WAN_SHI_TONG,
        key: "WAN_SHI_TONG",
        name: "Wan Shi Tong",
    },
    CardCatalogEntry {
        id: ids::WANDERING_ARCHAIC,
        key: "WANDERING_ARCHAIC",
        name: "Wandering Archaic",
    },
    CardCatalogEntry {
        id: ids::AN_OFFER_YOU_CANT_REFUSE,
        key: "AN_OFFER_YOU_CANT_REFUSE",
        name: "an Offer You Cant Refuse",
    },
    CardCatalogEntry {
        id: ids::CHORD_OF_CALLING,
        key: "CHORD_OF_CALLING",
        name: "Chord of Calling",
    },
    CardCatalogEntry {
        id: ids::CROP_ROTATION,
        key: "CROP_ROTATION",
        name: "Crop Rotation",
    },
    CardCatalogEntry {
        id: ids::CYCLONIC_RIFT,
        key: "CYCLONIC_RIFT",
        name: "Cyclonic Rift",
    },
    CardCatalogEntry {
        id: ids::FIERCE_GUARDIANSHIP,
        key: "FIERCE_GUARDIANSHIP",
        name: "Fierce Guardianship",
    },
    CardCatalogEntry {
        id: ids::FLUSTERSTORM,
        key: "FLUSTERSTORM",
        name: "Flusterstorm",
    },
    CardCatalogEntry {
        id: ids::FORCE_OF_NEGATION,
        key: "FORCE_OF_NEGATION",
        name: "Force of Negation",
    },
    CardCatalogEntry {
        id: ids::FORCE_OF_WILL,
        key: "FORCE_OF_WILL",
        name: "Force of Will",
    },
    CardCatalogEntry {
        id: ids::INTO_THE_FLOOD_MAW,
        key: "INTO_THE_FLOOD_MAW",
        name: "Into the Flood Maw",
    },
    CardCatalogEntry {
        id: ids::MENTAL_MISSTEP,
        key: "MENTAL_MISSTEP",
        name: "Mental Misstep",
    },
    CardCatalogEntry {
        id: ids::MINDBREAK_TRAP,
        key: "MINDBREAK_TRAP",
        name: "Mindbreak Trap",
    },
    CardCatalogEntry {
        id: ids::MYSTICAL_TUTOR,
        key: "MYSTICAL_TUTOR",
        name: "Mystical Tutor",
    },
    CardCatalogEntry {
        id: ids::NOXIOUS_REVIVAL,
        key: "NOXIOUS_REVIVAL",
        name: "Noxious Revival",
    },
    CardCatalogEntry {
        id: ids::PACT_OF_NEGATION,
        key: "PACT_OF_NEGATION",
        name: "Pact of Negation",
    },
    CardCatalogEntry {
        id: ids::SWAN_SONG,
        key: "SWAN_SONG",
        name: "Swan Song",
    },
    CardCatalogEntry {
        id: ids::VEIL_OF_SUMMER,
        key: "VEIL_OF_SUMMER",
        name: "Veil of Summer",
    },
    CardCatalogEntry {
        id: ids::WHIR_OF_INVENTION,
        key: "WHIR_OF_INVENTION",
        name: "Whir of Invention",
    },
    CardCatalogEntry {
        id: ids::WORLDLY_TUTOR,
        key: "WORLDLY_TUTOR",
        name: "Worldly Tutor",
    },
    CardCatalogEntry {
        id: ids::MUDDLE_THE_MIXTURE,
        key: "MUDDLE_THE_MIXTURE",
        name: "Muddle the Mixture",
    },
    CardCatalogEntry {
        id: ids::FINALE_OF_DEVASTATION,
        key: "FINALE_OF_DEVASTATION",
        name: "Finale of Devastation",
    },
    CardCatalogEntry {
        id: ids::GREEN_SUNS_ZENITH,
        key: "GREEN_SUNS_ZENITH",
        name: "Green Suns Zenith",
    },
    CardCatalogEntry {
        id: ids::NATURES_RHYTHM,
        key: "NATURES_RHYTHM",
        name: "Natures Rhythm",
    },
    CardCatalogEntry {
        id: ids::MYSTIC_REMORA,
        key: "MYSTIC_REMORA",
        name: "Mystic Remora",
    },
    CardCatalogEntry {
        id: ids::RHYSTIC_STUDY,
        key: "RHYSTIC_STUDY",
        name: "Rhystic Study",
    },
    CardCatalogEntry {
        id: ids::TEZZERET_THE_SEEKER,
        key: "TEZZERET_THE_SEEKER",
        name: "Tezzeret the Seeker",
    },
    CardCatalogEntry {
        id: ids::BRIDGEWORKS_BATTLE,
        key: "BRIDGEWORKS_BATTLE",
        name: "Bridgeworks Battle",
    },
    CardCatalogEntry {
        id: ids::DISCIPLE_OF_FREYALISE,
        key: "DISCIPLE_OF_FREYALISE",
        name: "Disciple of Freyalise",
    },
    CardCatalogEntry {
        id: ids::HYDROELECTRIC_SPECIMEN,
        key: "HYDROELECTRIC_SPECIMEN",
        name: "Hydroelectric Specimen",
    },
    CardCatalogEntry {
        id: ids::INVASION_OF_IKORIA,
        key: "INVASION_OF_IKORIA",
        name: "Invasion of Ikoria",
    },
    CardCatalogEntry {
        id: ids::SINK_INTO_STUPOR,
        key: "SINK_INTO_STUPOR",
        name: "Sink Into Stupor",
    },
];

/// Returns catalog entries annotated with whether the card currently has
/// any modeled spell/ability effect in the rules engine.
pub fn card_implementation_status(db: &CardDatabase) -> Vec<CardImplementationStatus> {
    ALL_CARDS
        .iter()
        .map(|entry| {
            let effects_implemented = db
                .get(entry.id)
                .map(|def| {
                    def.spell_effect.is_some()
                        || !def.activated_abilities.is_empty()
                        || !def.triggered_abilities.is_empty()
                        || !def.static_abilities.is_empty()
                })
                .unwrap_or(false);

            CardImplementationStatus {
                id: entry.id,
                key: entry.key,
                name: entry.name,
                effects_implemented,
            }
        })
        .collect()
}
