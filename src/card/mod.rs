pub mod sample;

use serde::{Deserialize, Serialize};
use std::fmt;

use crate::mana::{Color, ManaCost};

/// Unique identifier for a card definition (template).
pub type CardId = u64;

/// Unique identifier for a specific card instance in a game.
/// Two copies of "Lightning Bolt" share a CardId but have different ObjectId.
pub type ObjectId = u64;

/// Top-level card types in MTG.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CardType {
    Creature,
    Instant,
    Sorcery,
    Enchantment,
    Artifact,
    Planeswalker,
    Land,
}

/// Creature subtypes (a small representative set; extensible).
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Subtype(pub String);

/// Supertypes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Supertype {
    Basic,
    Legendary,
    Snow,
}

/// Keyword abilities that affect game rules directly.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum KeywordAbility {
    Flying,
    FirstStrike,
    DoubleStrike,
    Deathtouch,
    Haste,
    Hexproof,
    Indestructible,
    Lifelink,
    Menace,
    Reach,
    Trample,
    Vigilance,
    Defender,
    Flash,
    Fear,
    Intimidate,
    Shroud,
    Protection, // simplified — full protection needs a parameter
}

/// What kind of mana a land can produce.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ManaAbility {
    /// Tap to add one mana of the given color.
    TapForColor(Color),
    /// Tap to add one colorless mana.
    TapForColorless,
    /// Tap to add one mana of any color (e.g., City of Brass).
    TapForAny,
    /// Tap for one of several colors (e.g., dual lands).
    TapForChoice(Vec<Color>),
}

/// An activated ability (non-mana).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ActivatedAbility {
    pub cost: ManaCost,
    pub requires_tap: bool,
    pub effect: Effect,
    pub description: String,
}

/// A triggered ability.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TriggeredAbility {
    pub trigger: TriggerCondition,
    pub effect: Effect,
    pub description: String,
}

/// When a triggered ability fires.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TriggerCondition {
    EntersBattlefield,
    LeavesBattlefield,
    Dies,
    AttacksAlone,
    Attacks,
    BeginningOfUpkeep,
    EndOfTurn,
    DealsDamage,
    DealsCombatDamage,
    DealsCombatDamageToPlayer,
}

/// Effects that abilities and spells can produce.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Effect {
    DealDamage { amount: u32, target: TargetSpec },
    GainLife { amount: u32 },
    LoseLife { amount: u32, target: TargetSpec },
    DrawCards { count: u32 },
    DestroyTarget { target: TargetSpec },
    BounceTo { zone: ZoneType, target: TargetSpec },
    Buff { power: i32, toughness: i32, until_eot: bool },
    DiscardCards { count: u32, target: TargetSpec },
    CreateToken(TokenDef),
    Counter { target: TargetSpec },
    Multiple(Vec<Effect>),
    /// For effects we haven't modeled yet — described textually.
    Unimplemented(String),
}

/// What a targeting restriction looks like.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TargetSpec {
    /// Target any creature.
    AnyCreature,
    /// Target any player.
    AnyPlayer,
    /// Target creature or player.
    CreatureOrPlayer,
    /// Target creature or planeswalker.
    CreatureOrPlaneswalker,
    /// Target opponent.
    Opponent,
    /// Target the controller (self).
    Controller,
    /// Target any nonland permanent.
    AnyNonlandPermanent,
    /// Target any permanent.
    AnyPermanent,
    /// Target any spell on the stack.
    AnySpell,
    /// No target (e.g., "each opponent").
    NoTarget,
    /// Each creature on the battlefield (no targeting — affects all).
    EachCreature,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ZoneType {
    Library,
    Hand,
    Battlefield,
    Graveyard,
    Exile,
    Stack,
    Command,
}

/// Token creature definition.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TokenDef {
    pub name: String,
    pub power: u32,
    pub toughness: u32,
    pub colors: Vec<Color>,
    pub subtypes: Vec<Subtype>,
    pub keywords: Vec<KeywordAbility>,
}

/// The card definition — the "template" from which game objects are created.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CardDef {
    pub id: CardId,
    pub name: String,
    pub mana_cost: Option<ManaCost>,
    pub card_types: Vec<CardType>,
    pub supertypes: Vec<Supertype>,
    pub subtypes: Vec<Subtype>,
    pub keywords: Vec<KeywordAbility>,

    // Creature stats (None for non-creatures).
    pub power: Option<i32>,
    pub toughness: Option<i32>,

    // Land abilities.
    pub mana_abilities: Vec<ManaAbility>,

    // Spell/ability effects (for instants, sorceries, and ETB effects).
    pub spell_effect: Option<Effect>,

    // Activated and triggered abilities.
    pub activated_abilities: Vec<ActivatedAbility>,
    pub triggered_abilities: Vec<TriggeredAbility>,

    // Loyalty (planeswalkers).
    pub starting_loyalty: Option<u32>,

    // Whether this permanent enters the battlefield tapped.
    pub enters_tapped: bool,

    // Original oracle text for reference.
    pub oracle_text: String,
}

impl CardDef {
    pub fn is_creature(&self) -> bool {
        self.card_types.contains(&CardType::Creature)
    }

    pub fn is_land(&self) -> bool {
        self.card_types.contains(&CardType::Land)
    }

    pub fn is_instant(&self) -> bool {
        self.card_types.contains(&CardType::Instant)
    }

    pub fn is_sorcery(&self) -> bool {
        self.card_types.contains(&CardType::Sorcery)
    }

    pub fn has_flash(&self) -> bool {
        self.keywords.contains(&KeywordAbility::Flash)
    }

    pub fn is_basic_land(&self) -> bool {
        self.is_land() && self.supertypes.contains(&Supertype::Basic)
    }

    pub fn cmc(&self) -> u32 {
        self.mana_cost.as_ref().map(|c| c.cmc()).unwrap_or(0)
    }

    /// Can this card be cast at instant speed?
    pub fn is_instant_speed(&self) -> bool {
        self.is_instant() || self.has_flash()
    }

    /// Color identity of this card.
    pub fn color_identity(&self) -> Vec<Color> {
        self.mana_cost
            .as_ref()
            .map(|c| c.colors())
            .unwrap_or_default()
    }
}

impl fmt::Display for CardDef {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name)?;
        if let Some(ref cost) = self.mana_cost {
            write!(f, " {}", cost)?;
        }
        Ok(())
    }
}

/// A card object in the game — an instance of a CardDef with game-specific state.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CardInstance {
    pub object_id: ObjectId,
    pub card_def_id: CardId,
    pub owner: usize,     // player index
    pub controller: usize, // player index (may differ from owner)

    // Battlefield state
    pub tapped: bool,
    pub summoning_sick: bool,
    pub damage_marked: u32,
    pub plus_counters: i32,   // +1/+1 counters
    pub minus_counters: i32,  // -1/-1 counters

    // Temporary effects for the current turn
    pub temp_power_mod: i32,
    pub temp_toughness_mod: i32,
    pub temp_keywords: Vec<KeywordAbility>,

    // Attached permanents (auras, equipment).
    pub attached_to: Option<ObjectId>,
    pub attachments: Vec<ObjectId>,
}

impl CardInstance {
    pub fn new(object_id: ObjectId, card_def_id: CardId, owner: usize) -> Self {
        CardInstance {
            object_id,
            card_def_id,
            owner,
            controller: owner,
            tapped: false,
            summoning_sick: true,
            damage_marked: 0,
            plus_counters: 0,
            minus_counters: 0,
            temp_power_mod: 0,
            temp_toughness_mod: 0,
            temp_keywords: Vec::new(),
            attached_to: None,
            attachments: Vec::new(),
        }
    }

    /// Get effective power given the card definition.
    pub fn effective_power(&self, def: &CardDef) -> i32 {
        def.power.unwrap_or(0) + self.plus_counters - self.minus_counters + self.temp_power_mod
    }

    /// Get effective toughness given the card definition.
    pub fn effective_toughness(&self, def: &CardDef) -> i32 {
        def.toughness.unwrap_or(0) + self.plus_counters - self.minus_counters
            + self.temp_toughness_mod
    }

    /// Remaining toughness after damage.
    pub fn remaining_toughness(&self, def: &CardDef) -> i32 {
        self.effective_toughness(def) - self.damage_marked as i32
    }

    /// Check if this creature has a keyword (base + temporary).
    pub fn has_keyword(&self, def: &CardDef, kw: KeywordAbility) -> bool {
        def.keywords.contains(&kw) || self.temp_keywords.contains(&kw)
    }

    /// Can this creature attack? (not tapped, not sick unless haste, not defender)
    pub fn can_attack(&self, def: &CardDef) -> bool {
        !self.tapped
            && (!self.summoning_sick || self.has_keyword(def, KeywordAbility::Haste))
            && !self.has_keyword(def, KeywordAbility::Defender)
    }

    /// Reset end-of-turn temporary effects.
    pub fn cleanup_eot(&mut self) {
        self.temp_power_mod = 0;
        self.temp_toughness_mod = 0;
        self.temp_keywords.clear();
        self.damage_marked = 0;
    }
}

/// A deck list: card IDs and quantities.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Decklist {
    pub name: String,
    pub cards: Vec<DeckEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeckEntry {
    pub card_id: CardId,
    pub quantity: u32,
}

impl Decklist {
    pub fn total_cards(&self) -> u32 {
        self.cards.iter().map(|e| e.quantity).sum()
    }

    /// Expand into a flat list of CardIds (one per copy).
    pub fn expand(&self) -> Vec<CardId> {
        self.cards
            .iter()
            .flat_map(|e| std::iter::repeat(e.card_id).take(e.quantity as usize))
            .collect()
    }
}
