pub mod catalog;
pub mod sample;

use serde::{Deserialize, Serialize};
use std::fmt;

use crate::layers::StaticAbility;
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
    /// This creature must attack each combat if able (e.g., Juggernaut).
    MustAttack,
    /// This creature can't block (e.g., Goblin Guide).
    CantBlock,
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
    /// Tap to add N colorless mana (e.g., Sol Ring → 2, Basalt Monolith → 3).
    TapForColorlessAmount(u32),
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
    /// "Whenever a creature enters the battlefield" — watcher trigger (e.g., Soul Warden).
    /// Unlike EntersBattlefield (self-ETB), this fires for ANY creature entering.
    ACreatureEnters,
    LeavesBattlefield,
    Dies,
    /// "Whenever a creature dies" — watcher trigger (e.g., Blood Artist).
    /// Fires for ANY creature dying regardless of controller.
    ACreatureDies,
    /// "Whenever a creature you control dies" — filtered watcher trigger
    /// (e.g., Dictate of Erebos, Grave Pact). Only fires when the
    /// controller of the trigger source loses a creature.
    ACreatureYouControlDies,
    AttacksAlone,
    Attacks,
    BeginningOfUpkeep,
    /// "At the beginning of combat on your turn" (e.g., Lord Skitter).
    BeginningOfCombat,
    EndOfTurn,
    DealsDamage,
    DealsCombatDamage,
    DealsCombatDamageToPlayer,
    /// Whenever the controller casts any spell (e.g., Tidespout Tyrant).
    YouCastSpell,
    /// Whenever the controller casts a creature spell (e.g., Bontu's Monument).
    YouCastCreatureSpell,
    /// Whenever an opponent casts a noncreature spell (e.g., Mystic Remora, Nezahal).
    OpponentCastsNoncreatureSpell,
    /// Whenever an opponent casts any spell (e.g., Rhystic Study).
    OpponentCastsSpell,
    /// Whenever an opponent draws a card (e.g., Consecrated Sphinx).
    OpponentDrawsCard,
}

/// A dynamic value that can be computed at runtime from the game state.
/// Used for creatures with variable power/toughness like Tarmogoyf
/// ("*/1+* where * is the number of card types in all graveyards")
/// or Maro ("*/*, where * is the number of cards in your hand").
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum DynamicValue {
    /// Number of cards in the controller's hand (e.g., Maro).
    CardsInHand,
    /// Number of creatures the controller controls (e.g., Coat of Arms).
    CreaturesControlled,
    /// Number of card types among all graveyards (e.g., Tarmogoyf).
    CardTypesInGraveyards,
    /// Total power among creatures the controller controls.
    TotalPowerControlled,
    /// Number of Swamps the controller controls (e.g., Cabal Coffers).
    SwampsControlled,
    /// Devotion to a color — count of mana symbols of that color among
    /// permanents the controller controls (e.g., Nykthos, Shrine to Nyx).
    DevotionTo(Color),
    /// Number of creature cards in the controller's graveyard (e.g., Crypt of Agadeem).
    CreaturesInGraveyard,
    /// A fixed value (for testing / compatibility).
    Fixed(i32),
}

/// Extra context from the game state for evaluating `DynamicValue` variants
/// that need data beyond the battlefield (hand size, graveyards).
pub struct DynamicContext {
    /// Number of cards in the controller's hand.
    pub hand_size: usize,
    /// All card type sets across all graveyards, flattened for counting
    /// distinct types. Each inner Vec is one card's types.
    pub graveyard_card_types: Vec<Vec<CardType>>,
    /// Number of creature cards in the controller's graveyard.
    pub creatures_in_graveyard: usize,
}

impl DynamicValue {
    /// Evaluate this dynamic value in context.
    /// `controller` is the controlling player index, `objects` is the full
    /// object map, `battlefield` the set of object IDs on the battlefield,
    /// `card_db` looks up card definitions by ID, and `ctx` provides
    /// player hand/graveyard data needed by some variants.
    pub fn evaluate<'a, F>(
        &self,
        controller: usize,
        objects: &std::collections::HashMap<ObjectId, CardInstance>,
        battlefield: &[ObjectId],
        card_db: &'a F,
        ctx: Option<&DynamicContext>,
    ) -> i32
    where
        F: Fn(u64) -> Option<&'a CardDef>,
    {
        match self {
            DynamicValue::CardsInHand => ctx.map(|c| c.hand_size as i32).unwrap_or(0),
            DynamicValue::CreaturesControlled => battlefield
                .iter()
                .filter(|&&id| {
                    if let Some(inst) = objects.get(&id) {
                        if inst.controller != controller {
                            return false;
                        }
                        if let Some(def) = card_db(inst.card_def_id) {
                            return def.is_creature();
                        }
                    }
                    false
                })
                .count() as i32,
            DynamicValue::CardTypesInGraveyards => {
                if let Some(c) = ctx {
                    let mut seen = std::collections::HashSet::new();
                    for types in &c.graveyard_card_types {
                        for ct in types {
                            seen.insert(*ct);
                        }
                    }
                    seen.len() as i32
                } else {
                    0
                }
            }
            DynamicValue::TotalPowerControlled => battlefield
                .iter()
                .filter_map(|&id| {
                    let inst = objects.get(&id)?;
                    if inst.controller != controller {
                        return None;
                    }
                    let def = card_db(inst.card_def_id)?;
                    if def.is_creature() {
                        def.power
                    } else {
                        None
                    }
                })
                .sum(),
            DynamicValue::SwampsControlled => {
                battlefield
                    .iter()
                    .filter(|&&id| {
                        if let Some(inst) = objects.get(&id) {
                            if inst.controller != controller {
                                return false;
                            }
                            if let Some(def) = card_db(inst.card_def_id) {
                                return def.is_land()
                                    && def.subtypes.iter().any(|s| s.0 == "Swamp");
                            }
                        }
                        false
                    })
                    .count() as i32
            }
            DynamicValue::DevotionTo(color) => {
                battlefield
                    .iter()
                    .filter_map(|&id| {
                        let inst = objects.get(&id)?;
                        if inst.controller != controller {
                            return None;
                        }
                        let def = card_db(inst.card_def_id)?;
                        def.mana_cost.as_ref().map(|cost| cost.color_amount(*color))
                    })
                    .sum::<u32>() as i32
            }
            DynamicValue::CreaturesInGraveyard => {
                ctx.map(|c| c.creatures_in_graveyard as i32).unwrap_or(0)
            }
            DynamicValue::Fixed(val) => *val,
        }
    }
}

/// Effects that abilities and spells can produce.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Effect {
    DealDamage {
        amount: u32,
        target: TargetSpec,
    },
    GainLife {
        amount: u32,
    },
    LoseLife {
        amount: u32,
        target: TargetSpec,
    },
    DrawCards {
        count: u32,
    },
    DestroyTarget {
        target: TargetSpec,
    },
    /// Exile target (e.g., Swords to Plowshares, Path to Exile).
    ExileTarget {
        target: TargetSpec,
    },
    /// Destroy all creatures (e.g., Wrath of God, Day of Judgment).
    DestroyAll,
    BounceTo {
        zone: ZoneType,
        target: TargetSpec,
    },
    Buff {
        power: i32,
        toughness: i32,
        until_eot: bool,
    },
    /// Debuff: target creature gets -N/-N until end of turn.
    Debuff {
        power: i32,
        toughness: i32,
        until_eot: bool,
    },
    DiscardCards {
        count: u32,
        target: TargetSpec,
    },
    CreateToken(TokenDef),
    /// Create N tokens where N is determined by a dynamic value at runtime.
    /// Used for effects like Marrow-Gnawer ("Create X 1/1 Rat tokens, where
    /// X is the number of Rats you control").
    CreateTokens {
        token: TokenDef,
        count: DynamicValue,
    },
    Counter {
        target: TargetSpec,
    },
    /// Put +1/+1 counters on target creature.
    PutCounters {
        count: i32,
        target: TargetSpec,
    },
    /// Each player mills N cards.
    MillCards {
        count: u32,
        target: TargetSpec,
    },
    /// Each player sacrifices N creatures.
    SacrificeCreatures {
        count: u32,
        target: TargetSpec,
    },
    /// Prevent all combat damage this turn.
    PreventCombatDamage,
    /// Add mana to the controller's mana pool.
    AddMana {
        color: Option<Color>,
        amount: u32,
    },
    /// Add mana where the amount is determined dynamically at runtime
    /// (e.g., Cabal Coffers: "{B} for each Swamp you control").
    AddDynamicMana {
        color: Color,
        count: DynamicValue,
    },
    /// Lose life where the amount is determined dynamically
    /// (e.g., Castle Locthwain: "lose life equal to cards in hand").
    LoseDynamicLife {
        amount: DynamicValue,
        target: TargetSpec,
    },
    /// Take an extra turn after this one (e.g., Time Walk, Temporal Manipulation).
    ExtraTurn,
    /// Skip a phase of the controller's next turn (e.g., Stasis skipping untap).
    SkipPhase(crate::game::Phase),
    Multiple(Vec<Effect>),
    /// Search the controller's library and put a card into the destination zone.
    /// Simplified tutor — in practice the strategy chooses; the engine just moves
    /// the top matching card.
    SearchLibrary {
        destination: ZoneType,
    },
    /// Bounce all nonland permanents opponents control (e.g., Cyclonic Rift overload).
    BounceAllNonlandOpponents,
    /// Put a card from a graveyard on top of its owner's library.
    ReturnToTopOfLibrary {
        target: TargetSpec,
    },
    /// Untap target permanent.
    UntapTarget {
        target: TargetSpec,
    },
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

/// What spells a cost reduction applies to.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CostReductionTarget {
    /// All spells (e.g., Jet Medallion for black spells in mono-black).
    AllSpells,
    /// Only creature spells (e.g., Urza's Incubator, Herald's Horn, Bontu's Monument).
    CreatureSpells,
}

/// Cost reduction provided by a permanent on the battlefield.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CostReduction {
    /// How much generic mana to reduce.
    pub generic_reduction: u32,
    /// What spells the reduction applies to.
    pub applies_to: CostReductionTarget,
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

    // Static abilities that generate continuous effects on the battlefield.
    pub static_abilities: Vec<StaticAbility>,

    // Loyalty (planeswalkers).
    pub starting_loyalty: Option<u32>,

    // Whether this permanent enters the battlefield tapped.
    pub enters_tapped: bool,

    // Original oracle text for reference.
    pub oracle_text: String,

    /// Dynamic power formula — if set, the creature's base power is computed
    /// at runtime from the game state (e.g., Tarmogoyf, Maro).
    /// Takes precedence over the `power` field when present.
    #[serde(default)]
    pub dynamic_power: Option<DynamicValue>,

    /// Dynamic toughness formula — same as dynamic_power but for toughness.
    #[serde(default)]
    pub dynamic_toughness: Option<DynamicValue>,

    /// Cost reduction this permanent provides while on the battlefield
    /// (e.g., Jet Medallion: black spells cost {1} less).
    #[serde(default)]
    pub cost_reduction: Option<CostReduction>,
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

    /// Color identity of this card (CR 903.4).
    ///
    /// Includes colors from the mana cost and from mana abilities (basic land
    /// subtypes, dual lands, etc.). Full rules text scanning for mana symbols
    /// is not yet implemented.
    pub fn color_identity(&self) -> Vec<Color> {
        let mut colors = std::collections::HashSet::new();
        // Mana cost contributes to color identity
        if let Some(ref cost) = self.mana_cost {
            for c in cost.colors() {
                colors.insert(c);
            }
        }
        // Mana abilities contribute to color identity (basic land subtypes, duals)
        for ability in &self.mana_abilities {
            match ability {
                ManaAbility::TapForColor(c) => {
                    colors.insert(*c);
                }
                ManaAbility::TapForChoice(cs) => {
                    for c in cs {
                        colors.insert(*c);
                    }
                }
                _ => {}
            }
        }
        let mut result: Vec<Color> = colors.into_iter().collect();
        result.sort_by_key(|c| *c as u8);
        result
    }
}

impl Default for CardDef {
    fn default() -> Self {
        CardDef {
            id: 0,
            name: String::new(),
            mana_cost: None,
            card_types: Vec::new(),
            supertypes: Vec::new(),
            subtypes: Vec::new(),
            keywords: Vec::new(),
            power: None,
            toughness: None,
            mana_abilities: Vec::new(),
            spell_effect: None,
            activated_abilities: Vec::new(),
            triggered_abilities: Vec::new(),
            static_abilities: Vec::new(),
            starting_loyalty: None,
            enters_tapped: false,
            oracle_text: String::new(),
            dynamic_power: None,
            dynamic_toughness: None,
            cost_reduction: None,
        }
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
    pub owner: usize,      // player index
    pub controller: usize, // player index (may differ from owner)

    // Battlefield state
    pub tapped: bool,
    pub summoning_sick: bool,
    pub damage_marked: u32,
    pub plus_counters: i32,  // +1/+1 counters
    pub minus_counters: i32, // -1/-1 counters

    // Temporary effects for the current turn
    pub temp_power_mod: i32,
    pub temp_toughness_mod: i32,
    pub temp_keywords: Vec<KeywordAbility>,

    // Attached permanents (auras, equipment).
    pub attached_to: Option<ObjectId>,
    pub attachments: Vec<ObjectId>,

    /// Whether this is a token (CR 111.6). Tokens cease to exist when
    /// they leave the battlefield (CR 111.7).
    pub is_token: bool,

    /// Zone-change counter — incremented each time this object changes zones.
    /// Used to detect stale references (e.g., targeting a creature that has left
    /// and re-entered the battlefield is a different game object per CR 400.7).
    pub zone_change_count: u32,
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
            is_token: false,
            zone_change_count: 0,
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
    /// Commander cards (for Commander format decks).
    #[serde(default)]
    pub commanders: Vec<DeckEntry>,
    /// Cards that tutors can search for. When a tutor effect resolves and this
    /// list is non-empty, the player chooses from this restricted set (intersected
    /// with cards actually in their library). MCCFR learns which target is optimal
    /// in each game state.
    ///
    /// If empty, tutors fall back to taking the top card of the library (legacy
    /// behavior), keeping backward compatibility with existing tests/configs.
    #[serde(default)]
    pub tutor_targets: Vec<CardId>,
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
