use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::{CardDef, CardInstance, CardType, KeywordAbility, ObjectId, Subtype, ZoneType};
use crate::mana::Color;

/// A dynamic value that can be computed at runtime from the game state.
/// Used for creatures with variable power/toughness like Tarmogoyf
/// ("*/1+* where * is the number of card types in all graveyards")
/// or Maro ("*/*, where * is the number of cards in your hand").
///
/// # Implementation Status
/// - No annotation = evaluate() branch implemented
/// - `// UNIMPLEMENTED` = variant declared; evaluate() returns 0 for it
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
    /// Number of creatures with a specific subtype the controller controls
    /// (e.g., "number of Rats you control").
    CreaturesWithSubtype(String),
    /// A fixed value (for testing / compatibility).
    Fixed(i32),

    // ------------------------------------------------------------------
    // Additional dynamic values — UNIMPLEMENTED (evaluate() returns 0)
    // ------------------------------------------------------------------

    /// Number of artifacts the controller controls (e.g., Tezzeret effects). // UNIMPLEMENTED
    ArtifactsControlled,
    /// Number of enchantments the controller controls.                        // UNIMPLEMENTED
    EnchantmentsControlled,
    /// Number of lands the controller controls.                               // UNIMPLEMENTED
    LandsControlled,
    /// Total number of permanents the controller controls.                    // UNIMPLEMENTED
    PermanentsControlled,
    /// Number of spells cast by the controller this turn (e.g., Storm count). // UNIMPLEMENTED
    SpellsCastThisTurn,
    /// Number of +1/+1 (or other) counters on the source permanent.          // UNIMPLEMENTED
    CountersOnSource,
    /// Number of lands of a named basic type the controller controls
    /// (generalization of SwampsControlled — e.g., "Forests" for Arbor Elf). // UNIMPLEMENTED
    LandsOfType(String),
    /// Number of cards in all graveyards (yours + opponents').                // UNIMPLEMENTED
    AllGraveyardSize,
    /// Number of cards in the controller's graveyard.                        // UNIMPLEMENTED
    OwnGraveyardSize,
    /// The controller's current life total (e.g., Aetherflux Reservoir).    // UNIMPLEMENTED
    OwnLifeTotal,
    /// The number of opponents (useful for scaling Commander effects).       // UNIMPLEMENTED
    OpponentCount,
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
    pub fn evaluate<'a, F>(
        &self,
        controller: usize,
        objects: &HashMap<ObjectId, CardInstance>,
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
            DynamicValue::CreaturesWithSubtype(subtype_name) => battlefield
                .iter()
                .filter(|&&id| {
                    if let Some(inst) = objects.get(&id) {
                        if inst.controller != controller {
                            return false;
                        }
                        if let Some(def) = card_db(inst.card_def_id) {
                            return def.is_creature()
                                && def.subtypes.iter().any(|s| s.0 == *subtype_name);
                        }
                    }
                    false
                })
                .count() as i32,
            DynamicValue::Fixed(val) => *val,
            // --- Unimplemented variants: return 0 as a safe default ---
            DynamicValue::ArtifactsControlled
            | DynamicValue::EnchantmentsControlled
            | DynamicValue::LandsControlled
            | DynamicValue::PermanentsControlled
            | DynamicValue::SpellsCastThisTurn
            | DynamicValue::CountersOnSource
            | DynamicValue::AllGraveyardSize
            | DynamicValue::OwnGraveyardSize
            | DynamicValue::OwnLifeTotal
            | DynamicValue::OpponentCount
            | DynamicValue::LandsOfType(_) => 0,
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

    // --- Scry / library manipulation ---

    /// Scry N — look at top N cards, put any on bottom in any order, rest on top.
    /// PARTIAL: simplified in goldfish mode (no player choice; cards left in place).
    Scry {
        count: u32,
    },

    /// Surveil N — look at top N cards, put any into your graveyard, rest on top.
    /// Similar to Scry but cards can go to the graveyard.                    // UNIMPLEMENTED
    Surveil {
        count: u32,
    },

    /// Look at the top N cards of your library (without rearranging).        // UNIMPLEMENTED
    LookAtTopN {
        count: u32,
    },

    /// Discover N — exile cards from the top of your library until you exile
    /// a nonland card with mana value N or less; you may cast it for free,
    /// then put the rest on the bottom in a random order.                     // UNIMPLEMENTED
    Discover {
        value: u32,
    },

    /// Proliferate — for each permanent/player with a counter, add one more of that type.
    Proliferate,

    // --- Copy effects ---

    /// Copy target spell on the stack (e.g., Twincast, Fork).               // UNIMPLEMENTED
    CopySpell {
        target: TargetSpec,
    },

    /// Create a token that's a copy of target permanent (e.g., Rite of Replication). // UNIMPLEMENTED
    CopyPermanent {
        target: TargetSpec,
    },

    // --- Transform / face-down effects ---

    /// Transform target double-faced permanent (e.g., Delver of Secrets flip). // UNIMPLEMENTED
    TransformPermanent {
        target: TargetSpec,
    },

    /// Phase out target permanent (e.g., Teferi's Protection).              // UNIMPLEMENTED
    PhaseOut {
        target: TargetSpec,
    },

    /// Exile target and return it to the battlefield at the beginning of the
    /// next end step (flicker effect; e.g., Conjurer's Closet, Eerie Interlude). // UNIMPLEMENTED
    ExileAndReturnAtEOT {
        target: TargetSpec,
    },

    // --- Token / counter keywords as effects ---

    /// Populate — copy a creature token you control.                          // UNIMPLEMENTED
    Populate,

    /// Investigate — create a Clue artifact token.                           // UNIMPLEMENTED
    /// (Convenience shorthand; equivalent to CreatePredefinedToken { Clue, 1 }
    /// but makes the mechanic name explicit.)
    Investigate {
        count: u32,
    },

    /// Explore — target creature explores: reveal the top card of your library;
    /// if it's a land, put it into your hand; otherwise, put a +1/+1 counter
    /// on this creature, then you may put that card in your graveyard.       // UNIMPLEMENTED
    Explore {
        target: TargetSpec,
    },

    /// Adapt N — if this creature has no +1/+1 counters on it, put N +1/+1
    /// counters on it.                                                        // UNIMPLEMENTED
    Adapt {
        n: u32,
    },

    /// Amass Zombies N — put N +1/+1 counters on an Army token you control
    /// (or create a 0/0 black Zombie Army token first if you control none).  // UNIMPLEMENTED
    AmassZombies {
        n: u32,
    },

    /// Monstrous N — if this creature isn't monstrous, put N +1/+1 counters
    /// on it and it becomes monstrous.                                        // UNIMPLEMENTED
    Monstrous {
        n: u32,
    },

    // --- Counter / poison counters ---

    /// Add N poison counters to target player.                               // UNIMPLEMENTED
    AddPoisonCounters {
        count: u32,
        target: TargetSpec,
    },

    /// Add N energy counters to the controller (Kaladesh block mechanic).   // UNIMPLEMENTED
    AddEnergyCounters {
        count: u32,
    },

    /// Pay N energy counters (part of energy-based effects).                 // UNIMPLEMENTED
    PayEnergyCounters {
        count: u32,
    },

    /// Add N experience counters to the controller (Commander 2015).        // UNIMPLEMENTED
    AddExperienceCounters {
        count: u32,
    },

    // --- Regeneration ---

    /// Regenerate target creature — the next time it would be destroyed this
    /// turn, tap it and remove it from combat instead.                       // UNIMPLEMENTED
    Regenerate {
        target: TargetSpec,
    },

    // --- Combat modifiers ---

    /// Prevent all damage that would be dealt to and by target creature
    /// this turn (e.g., Fog of War).                                         // UNIMPLEMENTED
    PreventAllDamageToTarget {
        target: TargetSpec,
    },

    /// Prevent all combat damage this turn (e.g., Fog, Holy Day).
    /// PARTIAL: declared; not applied during damage calculation.
    PreventCombatDamage,

    /// Redirect damage that would be dealt to target player to instead be
    /// dealt to another target (e.g., Misdirection-style effects).           // UNIMPLEMENTED
    RedirectDamage {
        amount: u32,
        from: TargetSpec,
        to: TargetSpec,
    },

    // --- Zone: put from hand / graveyard face-down ---

    /// Manifest N — put the top N cards of your library onto the battlefield
    /// face-down as 2/2 creatures.                                           // UNIMPLEMENTED
    Manifest {
        count: u32,
    },

    // --- Monarch / initiative ---

    /// Become the monarch (or give the monarchy to target player).           // UNIMPLEMENTED
    BecomeMonarch,

    /// Take the initiative (sets the initiative marker; you go to the first
    /// room of the dungeon on your upkeep).                                  // UNIMPLEMENTED
    TakeInitiative,

    /// Venture into the dungeon — advance through one room in the dungeon.  // UNIMPLEMENTED
    Venture,

    // --- Learn ---

    /// Learn — look at the top card of your sideboard and reveal it, then
    /// put it into your hand or discard a card to draw a card.               // UNIMPLEMENTED
    Learn,

    // --- Coin flip ---

    /// Flip a coin: execute on_heads or on_tails depending on the result
    /// (e.g., Krark's Thumb, Zndrsplt).                                     // UNIMPLEMENTED
    FlipCoin {
        on_heads: Box<Effect>,
        on_tails: Box<Effect>,
    },

    // --- Add mana ---

    /// Add mana to the controller's mana pool.
    AddMana {
        color: Option<Color>,
        amount: u32,
    },

    /// Add mana where the amount is determined dynamically at runtime.
    AddDynamicMana {
        color: Color,
        count: DynamicValue,
    },

    // --- Misc player effects ---

    /// Lose life where the amount is determined dynamically.
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
    SearchLibrary {
        destination: ZoneType,
        subtype_filter: Vec<Subtype>,
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

    // --- Zone manipulation effects ---

    /// Return target card from graveyard to battlefield (e.g., Reanimate, Animate Dead).
    ReturnFromGraveyardToBattlefield {
        target: TargetSpec,
    },

    /// Return target card from graveyard to hand (e.g., Regrowth, Eternal Witness).
    ReturnFromGraveyardToHand {
        target: TargetSpec,
    },

    /// Exile target card from a graveyard (e.g., Bojuka Bog, Tormod's Crypt).
    ExileFromGraveyard {
        target: TargetSpec,
    },

    /// Shuffle target(s) into their owner's library.
    ShuffleIntoLibrary {
        target: TargetSpec,
    },

    /// Put a card on the bottom of its owner's library (e.g., Terminus, Hinder).
    PutOnBottomOfLibrary {
        target: TargetSpec,
    },

    // --- Creature/permanent manipulation ---

    /// Target creature gains a keyword ability until end of turn.
    GainKeywordUntilEOT {
        keyword: KeywordAbility,
        target: TargetSpec,
    },

    /// Set target creature's base power and toughness (e.g., Turn to Frog, Humility).
    SetPowerToughness {
        power: i32,
        toughness: i32,
        until_eot: bool,
        target: TargetSpec,
    },

    /// Gain control of target permanent until end of turn (e.g., Act of Treason).
    GainControlUntilEOT {
        target: TargetSpec,
    },

    /// Gain permanent control of target permanent (e.g., Confiscate, Control Magic). // UNIMPLEMENTED
    GainControlPermanent {
        target: TargetSpec,
    },

    /// Target creature fights another target creature (e.g., Prey Upon, Domri Rade).
    Fight {
        target: TargetSpec,
    },

    /// Tap target permanent (e.g., Frost Breath, Icy Manipulator).
    TapTarget {
        target: TargetSpec,
    },

    /// Untap all creatures you control (e.g., Seedborn Muse trigger).       // UNIMPLEMENTED
    UntapAllYouControl,

    /// Give target permanent indestructible until end of turn
    /// (e.g., Heroic Intervention, Teferi's Protection).                     // UNIMPLEMENTED
    GainIndestructibleUntilEOT {
        target: TargetSpec,
    },

    /// Attach this Equipment/Aura to target creature you control.            // UNIMPLEMENTED
    AttachTo {
        target: TargetSpec,
    },

    // --- Player-targeted effects ---

    /// Each opponent loses N life (e.g., Blood Artist, Gray Merchant of Asphodel).
    EachOpponentLosesLife {
        amount: u32,
    },

    /// Each opponent discards N cards (e.g., Sire of Insanity, Bottomless Pit).
    EachOpponentDiscards {
        count: u32,
    },

    /// Each opponent sacrifices N creatures (e.g., Fleshbag Marauder, Dictate of Erebos).
    EachOpponentSacrifices {
        count: u32,
    },

    /// Target player draws N cards then discards M cards (e.g., Faithless Looting).
    DrawThenDiscard {
        draw: u32,
        discard: u32,
        target: TargetSpec,
    },

    /// Gain life equal to a dynamic value (e.g., Gray Merchant drains for devotion).
    GainDynamicLife {
        amount: DynamicValue,
    },

    /// Each player draws N cards (e.g., Howling Mine, Kami of the Crescent Moon). // UNIMPLEMENTED
    EachPlayerDraws {
        count: u32,
    },

    /// Each player discards their hand and draws N cards
    /// (e.g., Windfall, Timetwister).                                        // UNIMPLEMENTED
    EachPlayerDiscardsAndDraws {
        draw: u32,
    },

    // --- Conditional/modal effects ---

    /// Choose one (or more) from a list of effects.
    /// PARTIAL: AI always chooses the first N; no strategic evaluation.
    Modal {
        choices: Vec<Effect>,
        choose_count: u32,
    },

    /// Execute an effect only if a condition is true; otherwise execute the else branch.
    Conditional {
        condition: Condition,
        if_true: Box<Effect>,
        if_false: Option<Box<Effect>>,
    },

    /// Repeat an effect for each of a variable (e.g., "for each creature you control").
    ForEach {
        count: DynamicValue,
        effect: Box<Effect>,
    },

    // --- Predefined token shortcuts ---

    /// Create a predefined token type (Treasure, Food, Clue, Blood, etc.).
    CreatePredefinedToken {
        token_type: PredefinedToken,
        count: u32,
    },

    /// Buff other creatures with a specific subtype you control by +X/+X
    /// until end of turn, where X is a dynamic value.
    /// (e.g., Ashcoat: "other Rats you control get +X/+X where X = rats you control")
    BuffOtherSubtype {
        subtype: String,
        amount: DynamicValue,
        until_eot: bool,
    },

    /// For effects we haven't modeled yet — described textually.
    Unimplemented(String),
}

/// Conditions that can be checked at runtime for conditional effects.
///
/// # Implementation Status
/// - No annotation = check_condition() fully implemented
/// - `// UNIMPLEMENTED` = declared; check_condition() returns false for it
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Condition {
    /// Controller controls at least one creature.
    ControlCreatures,
    /// Controller's life is at or above N.
    LifeAtOrAbove(i32),
    /// Controller's life is at or below N.
    LifeAtOrBelow(i32),
    /// It is the controller's turn.
    IsYourTurn,
    /// The source permanent has +1/+1 counters.
    SourceHasCounters,
    /// Controller controls N or more permanents of a type.
    ControlNOrMore {
        count: u32,
        card_type: CardType,
    },
    /// Always true (for testing / default).
    Always,

    // ------------------------------------------------------------------
    // Additional conditions — UNIMPLEMENTED (check_condition() returns false)
    // ------------------------------------------------------------------

    /// It is an opponent's turn.                                              // UNIMPLEMENTED
    IsOpponentsTurn,
    /// Controller's hand is empty (e.g., Hellbent).                          // UNIMPLEMENTED
    HandIsEmpty,
    /// Controller's graveyard has N or more cards (e.g., Threshold: 7+).     // UNIMPLEMENTED
    GraveyardHasNCards(u32),
    /// Controller controls at least one artifact.                             // UNIMPLEMENTED
    ControlsArtifact,
    /// Controller controls at least one enchantment.                          // UNIMPLEMENTED
    ControlsEnchantment,
    /// Controller controls at least one land of the named basic type.        // UNIMPLEMENTED
    ControlsLandType(String),
    /// An opponent has life at or below N (e.g., Knight of the Ebon Legion). // UNIMPLEMENTED
    OpponentLifeAtOrBelow(i32),
    /// The permanent is attacking this turn.                                  // UNIMPLEMENTED
    IsAttacking,
    /// The permanent is blocking this turn.                                   // UNIMPLEMENTED
    IsBlocking,
    /// The controller has the city's blessing (Ascend).                      // UNIMPLEMENTED
    HasCitysBlessing,
    /// The controller has the monarch.                                        // UNIMPLEMENTED
    HasMonarch,
    /// The target permanent has a counter of any kind on it.                 // UNIMPLEMENTED
    TargetHasCounter,
    /// An opponent controls more permanents of this type than the controller. // UNIMPLEMENTED
    OpponentControlsMore { card_type: CardType },
    /// The controller has cast a spell with mana value N or more this turn.  // UNIMPLEMENTED
    CastHighValueSpellThisTurn(u32),
    /// It is the first turn of the game (e.g., Chancellor of the Annex).     // UNIMPLEMENTED
    IsFirstTurn,
}

/// Predefined token types used across many cards.
///
/// # Implementation Status
/// - No annotation = to_token_def() fully implemented; token creates correctly
/// - `// UNIMPLEMENTED` = to_token_def() stub exists (0/0 token); ETB abilities not modeled
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PredefinedToken {
    // ------------------------------------------------------------------
    // Artifact tokens (fully implemented)
    // ------------------------------------------------------------------

    /// 0/0 colorless artifact — sacrifice: add one mana of any color.
    Treasure,
    /// 0/0 colorless artifact — sacrifice: gain 3 life.
    Food,
    /// 0/0 colorless artifact — sacrifice: draw a card.
    Clue,
    /// 0/0 colorless artifact — sacrifice: discard a card, then draw a card.
    Blood,

    // ------------------------------------------------------------------
    // Creature tokens (fully implemented)
    // ------------------------------------------------------------------

    /// 1/1 white Soldier creature.
    Soldier,
    /// 1/1 white Spirit creature with flying.
    Spirit,
    /// 2/2 black Zombie creature.
    Zombie,
    /// 1/1 green Saproling creature.
    Saproling,
    /// 1/1 red Goblin creature.
    Goblin,
    /// 1/1 white Human creature.
    Human,
    /// 3/3 green Beast creature.
    Beast,

    // ------------------------------------------------------------------
    // Additional creature tokens — UNIMPLEMENTED (to_token_def() returns
    // the correct stats; activated abilities not modeled)
    // ------------------------------------------------------------------

    /// 5/5 red Dragon creature with flying (e.g., Dragon Broodmother).       // UNIMPLEMENTED
    Dragon,
    /// 2/2 white Cat creature (e.g., Ajani Goldmane, Brimaz).               // UNIMPLEMENTED
    Cat,
    /// 1/1 black Rat creature (e.g., Pack Rat, Ashcoat).                     // UNIMPLEMENTED
    Rat,
    /// 1/1 green Elf Warrior creature (e.g., Freyalise, Elvish Promenade).   // UNIMPLEMENTED
    ElfWarrior,
    /// 1/1 white Warrior creature (e.g., Secure the Wastes).                 // UNIMPLEMENTED
    Warrior,
    /// 1/1 white Knight creature with vigilance (e.g., Elspeth, Sun's Champion).// UNIMPLEMENTED
    Knight,
    /// 1/1 green Insect creature (e.g., Hornet Queen has deathtouch too).    // UNIMPLEMENTED
    Insect,
    /// 1/2 green Spider creature with reach (e.g., Ishkanah).                // UNIMPLEMENTED
    Spider,
    /// 1/1 white Bird creature with flying (e.g., Lingering Souls).          // UNIMPLEMENTED
    Bird,
    /// 4/4 white Angel creature with flying and vigilance (e.g., Elspeth).   // UNIMPLEMENTED
    Angel,
    /// 2/2 green Wolf creature (e.g., Garruk Relentless, Immerwolf).         // UNIMPLEMENTED
    Wolf,
    /// 1/1 white Vampire creature with lifelink (e.g., Sorin).              // UNIMPLEMENTED
    Vampire,
    /// 1/1 black Skeleton creature (e.g., Field of the Dead).                // UNIMPLEMENTED
    Skeleton,
    /// 2/2 green Wurm creature (e.g., Advent of the Wurm).                   // UNIMPLEMENTED
    Wurm,
    /// 1/1 black and green Pest creature with "when dies, gain 1 life"
    /// (e.g., Witherbloom Apprentice).                                       // UNIMPLEMENTED
    Pest,
    /// 2/2 red Dragon Egg creature (e.g., Dragon Egg's ETB).                 // UNIMPLEMENTED
    DragonEgg,
    /// 0/1 colorless Eldrazi Spawn creature with
    /// "sacrifice: add {C}" (e.g., Emrakul's Hatcher).                       // UNIMPLEMENTED
    EldraziSpawn,
    /// 1/1 colorless Eldrazi Scion creature with
    /// "sacrifice: add {C}" (e.g., From Beyond).                             // UNIMPLEMENTED
    EldraziScion,
    /// 1/1 blue Faerie Rogue creature with flying (e.g., Bitterblossom).     // UNIMPLEMENTED
    FaerieRogue,
    /// 2/2 black Zombie Knight creature (e.g., Order of Midnight).           // UNIMPLEMENTED
    ZombieKnight,
    /// 1/1 red and white Dwarf creature (e.g., various Adventures in the
    /// Forgotten Realms cards).                                               // UNIMPLEMENTED
    Dwarf,
    /// 3/3 colorless Golem artifact creature (e.g., Blade Splicer).
    /// Note: Blade Splicer already creates Golem via custom TokenDef;
    /// this provides a convenient shorthand.                                  // UNIMPLEMENTED
    Golem,
    /// 3/3 green Elephant creature (e.g., Elephant token from Garruk).       // UNIMPLEMENTED
    Elephant,
    /// 2/2 blue Drake creature with flying (e.g., Talrand, Sky Summoner).    // UNIMPLEMENTED
    Drake,
    /// 1/1 blue Illusion creature (e.g., Jace, Memory Adept).               // UNIMPLEMENTED
    Illusion,
    /// 2/2 colorless Assembly-Worker artifact creature (e.g., Mishra).       // UNIMPLEMENTED
    AssemblyWorker,
    /// 2/2 red Ogre creature (e.g., various red cards).                      // UNIMPLEMENTED
    Ogre,
    /// 1/1 colorless Phyrexian Germ creature (Living Weapon Equipment base). // UNIMPLEMENTED
    PhyrexianGerm,
    /// 2/2 red Rebel creature (For Mirrodin! Equipment base).                // UNIMPLEMENTED
    Rebel,

    // ------------------------------------------------------------------
    // Artifact tokens — UNIMPLEMENTED
    // ------------------------------------------------------------------

    /// 0/0 colorless artifact — sacrifice and pay {1}: draw a card
    /// (Map token, from explore-adjacent effects).                            // UNIMPLEMENTED
    Map,
    /// 0/0 colorless artifact — sacrifice: add one mana of any color
    /// (Gold token; similar to Treasure but older, from Conspiracy).         // UNIMPLEMENTED
    Gold,
    /// 0/0 colorless artifact — sacrifice: draw a card (Junk token,
    /// functional synonym for Clue in some sets).                            // UNIMPLEMENTED
    Junk,
    /// 0/0 colorless artifact — tap: add {C} (Powerstone token, from
    /// The Brothers' War).                                                   // UNIMPLEMENTED
    Powerstone,
    /// Incubator token — a double-faced artifact that can be transformed
    /// into a Phyrexian creature (March of the Machine).                     // UNIMPLEMENTED
    Incubator,
    /// Walker token — a 2/2 colorless Construct artifact creature
    /// (from Tezzerets and similar).                                         // UNIMPLEMENTED
    Walker,
    /// 1/1 colorless Thopter artifact creature with flying
    /// (e.g., Thopter Spy Network, Breya).                                  // UNIMPLEMENTED
    Thopter,
    /// 1/1 colorless Servo artifact creature
    /// (e.g., Servo Exhibition, Toolcraft Exemplar).                         // UNIMPLEMENTED
    Servo,
    /// 0/0 colorless Construct artifact creature that enters with
    /// an X/X counter where X is equal to the number of cards in your hand.  // UNIMPLEMENTED
    Construct,
}

impl PredefinedToken {
    /// Convert a predefined token type into a concrete TokenDef.
    pub fn to_token_def(&self) -> TokenDef {
        match self {
            PredefinedToken::Treasure => TokenDef {
                name: "Treasure".to_string(),
                power: 0,
                toughness: 0,
                colors: vec![],
                subtypes: vec![Subtype("Treasure".to_string())],
                keywords: vec![],
            },
            PredefinedToken::Food => TokenDef {
                name: "Food".to_string(),
                power: 0,
                toughness: 0,
                colors: vec![],
                subtypes: vec![Subtype("Food".to_string())],
                keywords: vec![],
            },
            PredefinedToken::Clue => TokenDef {
                name: "Clue".to_string(),
                power: 0,
                toughness: 0,
                colors: vec![],
                subtypes: vec![Subtype("Clue".to_string())],
                keywords: vec![],
            },
            PredefinedToken::Blood => TokenDef {
                name: "Blood".to_string(),
                power: 0,
                toughness: 0,
                colors: vec![],
                subtypes: vec![Subtype("Blood".to_string())],
                keywords: vec![],
            },
            PredefinedToken::Soldier => TokenDef {
                name: "Soldier".to_string(),
                power: 1,
                toughness: 1,
                colors: vec![Color::White],
                subtypes: vec![Subtype("Soldier".to_string())],
                keywords: vec![],
            },
            PredefinedToken::Spirit => TokenDef {
                name: "Spirit".to_string(),
                power: 1,
                toughness: 1,
                colors: vec![Color::White],
                subtypes: vec![Subtype("Spirit".to_string())],
                keywords: vec![KeywordAbility::Flying],
            },
            PredefinedToken::Zombie => TokenDef {
                name: "Zombie".to_string(),
                power: 2,
                toughness: 2,
                colors: vec![Color::Black],
                subtypes: vec![Subtype("Zombie".to_string())],
                keywords: vec![],
            },
            PredefinedToken::Saproling => TokenDef {
                name: "Saproling".to_string(),
                power: 1,
                toughness: 1,
                colors: vec![Color::Green],
                subtypes: vec![Subtype("Saproling".to_string())],
                keywords: vec![],
            },
            PredefinedToken::Goblin => TokenDef {
                name: "Goblin".to_string(),
                power: 1,
                toughness: 1,
                colors: vec![Color::Red],
                subtypes: vec![Subtype("Goblin".to_string())],
                keywords: vec![],
            },
            PredefinedToken::Human => TokenDef {
                name: "Human".to_string(),
                power: 1,
                toughness: 1,
                colors: vec![Color::White],
                subtypes: vec![Subtype("Human".to_string())],
                keywords: vec![],
            },
            PredefinedToken::Beast => TokenDef {
                name: "Beast".to_string(),
                power: 3,
                toughness: 3,
                colors: vec![Color::Green],
                subtypes: vec![Subtype("Beast".to_string())],
                keywords: vec![],
            },
            // --- Unimplemented tokens: provide correct stats; abilities not modeled ---
            PredefinedToken::Dragon => TokenDef {
                name: "Dragon".to_string(),
                power: 5,
                toughness: 5,
                colors: vec![Color::Red],
                subtypes: vec![Subtype("Dragon".to_string())],
                keywords: vec![KeywordAbility::Flying],
            },
            PredefinedToken::Cat => TokenDef {
                name: "Cat".to_string(),
                power: 2,
                toughness: 2,
                colors: vec![Color::White],
                subtypes: vec![Subtype("Cat".to_string())],
                keywords: vec![],
            },
            PredefinedToken::Rat => TokenDef {
                name: "Rat".to_string(),
                power: 1,
                toughness: 1,
                colors: vec![Color::Black],
                subtypes: vec![Subtype("Rat".to_string())],
                keywords: vec![],
            },
            PredefinedToken::ElfWarrior => TokenDef {
                name: "Elf Warrior".to_string(),
                power: 1,
                toughness: 1,
                colors: vec![Color::Green],
                subtypes: vec![Subtype("Elf".to_string()), Subtype("Warrior".to_string())],
                keywords: vec![],
            },
            PredefinedToken::Warrior => TokenDef {
                name: "Warrior".to_string(),
                power: 1,
                toughness: 1,
                colors: vec![Color::White],
                subtypes: vec![Subtype("Warrior".to_string())],
                keywords: vec![],
            },
            PredefinedToken::Knight => TokenDef {
                name: "Knight".to_string(),
                power: 1,
                toughness: 1,
                colors: vec![Color::White],
                subtypes: vec![Subtype("Knight".to_string())],
                keywords: vec![KeywordAbility::Vigilance],
            },
            PredefinedToken::Insect => TokenDef {
                name: "Insect".to_string(),
                power: 1,
                toughness: 1,
                colors: vec![Color::Green],
                subtypes: vec![Subtype("Insect".to_string())],
                keywords: vec![],
            },
            PredefinedToken::Spider => TokenDef {
                name: "Spider".to_string(),
                power: 1,
                toughness: 2,
                colors: vec![Color::Green],
                subtypes: vec![Subtype("Spider".to_string())],
                keywords: vec![KeywordAbility::Reach],
            },
            PredefinedToken::Bird => TokenDef {
                name: "Bird".to_string(),
                power: 1,
                toughness: 1,
                colors: vec![Color::White],
                subtypes: vec![Subtype("Bird".to_string())],
                keywords: vec![KeywordAbility::Flying],
            },
            PredefinedToken::Angel => TokenDef {
                name: "Angel".to_string(),
                power: 4,
                toughness: 4,
                colors: vec![Color::White],
                subtypes: vec![Subtype("Angel".to_string())],
                keywords: vec![KeywordAbility::Flying, KeywordAbility::Vigilance],
            },
            PredefinedToken::Wolf => TokenDef {
                name: "Wolf".to_string(),
                power: 2,
                toughness: 2,
                colors: vec![Color::Green],
                subtypes: vec![Subtype("Wolf".to_string())],
                keywords: vec![],
            },
            PredefinedToken::Vampire => TokenDef {
                name: "Vampire".to_string(),
                power: 1,
                toughness: 1,
                colors: vec![Color::White],
                subtypes: vec![Subtype("Vampire".to_string())],
                keywords: vec![KeywordAbility::Lifelink],
            },
            PredefinedToken::Skeleton => TokenDef {
                name: "Skeleton".to_string(),
                power: 1,
                toughness: 1,
                colors: vec![Color::Black],
                subtypes: vec![Subtype("Skeleton".to_string())],
                keywords: vec![],
            },
            PredefinedToken::Wurm => TokenDef {
                name: "Wurm".to_string(),
                power: 3,
                toughness: 3,
                colors: vec![Color::Green],
                subtypes: vec![Subtype("Wurm".to_string())],
                keywords: vec![],
            },
            PredefinedToken::Pest => TokenDef {
                name: "Pest".to_string(),
                power: 1,
                toughness: 1,
                colors: vec![Color::Black, Color::Green],
                subtypes: vec![Subtype("Pest".to_string())],
                keywords: vec![],
            },
            PredefinedToken::DragonEgg => TokenDef {
                name: "Dragon Egg".to_string(),
                power: 0,
                toughness: 2,
                colors: vec![Color::Red],
                subtypes: vec![Subtype("Dragon".to_string()), Subtype("Egg".to_string())],
                keywords: vec![KeywordAbility::Defender],
            },
            PredefinedToken::EldraziSpawn => TokenDef {
                name: "Eldrazi Spawn".to_string(),
                power: 0,
                toughness: 1,
                colors: vec![],
                subtypes: vec![Subtype("Eldrazi".to_string()), Subtype("Spawn".to_string())],
                keywords: vec![],
            },
            PredefinedToken::EldraziScion => TokenDef {
                name: "Eldrazi Scion".to_string(),
                power: 1,
                toughness: 1,
                colors: vec![],
                subtypes: vec![Subtype("Eldrazi".to_string()), Subtype("Scion".to_string())],
                keywords: vec![],
            },
            PredefinedToken::FaerieRogue => TokenDef {
                name: "Faerie Rogue".to_string(),
                power: 1,
                toughness: 1,
                colors: vec![Color::Blue],
                subtypes: vec![Subtype("Faerie".to_string()), Subtype("Rogue".to_string())],
                keywords: vec![KeywordAbility::Flying],
            },
            PredefinedToken::ZombieKnight => TokenDef {
                name: "Zombie Knight".to_string(),
                power: 2,
                toughness: 2,
                colors: vec![Color::Black],
                subtypes: vec![Subtype("Zombie".to_string()), Subtype("Knight".to_string())],
                keywords: vec![],
            },
            PredefinedToken::Dwarf => TokenDef {
                name: "Dwarf".to_string(),
                power: 1,
                toughness: 1,
                colors: vec![Color::Red, Color::White],
                subtypes: vec![Subtype("Dwarf".to_string())],
                keywords: vec![],
            },
            PredefinedToken::Golem => TokenDef {
                name: "Golem".to_string(),
                power: 3,
                toughness: 3,
                colors: vec![],
                subtypes: vec![Subtype("Golem".to_string())],
                keywords: vec![],
            },
            PredefinedToken::Elephant => TokenDef {
                name: "Elephant".to_string(),
                power: 3,
                toughness: 3,
                colors: vec![Color::Green],
                subtypes: vec![Subtype("Elephant".to_string())],
                keywords: vec![],
            },
            PredefinedToken::Drake => TokenDef {
                name: "Drake".to_string(),
                power: 2,
                toughness: 2,
                colors: vec![Color::Blue],
                subtypes: vec![Subtype("Drake".to_string())],
                keywords: vec![KeywordAbility::Flying],
            },
            PredefinedToken::Illusion => TokenDef {
                name: "Illusion".to_string(),
                power: 1,
                toughness: 1,
                colors: vec![Color::Blue],
                subtypes: vec![Subtype("Illusion".to_string())],
                keywords: vec![],
            },
            PredefinedToken::AssemblyWorker => TokenDef {
                name: "Assembly-Worker".to_string(),
                power: 2,
                toughness: 2,
                colors: vec![],
                subtypes: vec![Subtype("Assembly-Worker".to_string())],
                keywords: vec![],
            },
            PredefinedToken::Ogre => TokenDef {
                name: "Ogre".to_string(),
                power: 2,
                toughness: 2,
                colors: vec![Color::Red],
                subtypes: vec![Subtype("Ogre".to_string())],
                keywords: vec![],
            },
            PredefinedToken::PhyrexianGerm => TokenDef {
                name: "Phyrexian Germ".to_string(),
                power: 0,
                toughness: 0,
                colors: vec![Color::Black],
                subtypes: vec![Subtype("Phyrexian".to_string()), Subtype("Germ".to_string())],
                keywords: vec![],
            },
            PredefinedToken::Rebel => TokenDef {
                name: "Rebel".to_string(),
                power: 2,
                toughness: 2,
                colors: vec![Color::Red],
                subtypes: vec![Subtype("Rebel".to_string())],
                keywords: vec![],
            },
            // Artifact tokens (unimplemented abilities)
            PredefinedToken::Map => TokenDef {
                name: "Map".to_string(),
                power: 0,
                toughness: 0,
                colors: vec![],
                subtypes: vec![Subtype("Map".to_string())],
                keywords: vec![],
            },
            PredefinedToken::Gold => TokenDef {
                name: "Gold".to_string(),
                power: 0,
                toughness: 0,
                colors: vec![],
                subtypes: vec![Subtype("Gold".to_string())],
                keywords: vec![],
            },
            PredefinedToken::Junk => TokenDef {
                name: "Junk".to_string(),
                power: 0,
                toughness: 0,
                colors: vec![],
                subtypes: vec![Subtype("Junk".to_string())],
                keywords: vec![],
            },
            PredefinedToken::Powerstone => TokenDef {
                name: "Powerstone".to_string(),
                power: 0,
                toughness: 0,
                colors: vec![],
                subtypes: vec![Subtype("Powerstone".to_string())],
                keywords: vec![],
            },
            PredefinedToken::Incubator => TokenDef {
                name: "Incubator".to_string(),
                power: 0,
                toughness: 0,
                colors: vec![],
                subtypes: vec![Subtype("Incubator".to_string())],
                keywords: vec![],
            },
            PredefinedToken::Walker => TokenDef {
                name: "Walker".to_string(),
                power: 2,
                toughness: 2,
                colors: vec![],
                subtypes: vec![Subtype("Construct".to_string())],
                keywords: vec![],
            },
            PredefinedToken::Thopter => TokenDef {
                name: "Thopter".to_string(),
                power: 1,
                toughness: 1,
                colors: vec![],
                subtypes: vec![Subtype("Thopter".to_string())],
                keywords: vec![KeywordAbility::Flying],
            },
            PredefinedToken::Servo => TokenDef {
                name: "Servo".to_string(),
                power: 1,
                toughness: 1,
                colors: vec![],
                subtypes: vec![Subtype("Servo".to_string())],
                keywords: vec![],
            },
            PredefinedToken::Construct => TokenDef {
                name: "Construct".to_string(),
                power: 0,
                toughness: 0,
                colors: vec![],
                subtypes: vec![Subtype("Construct".to_string())],
                keywords: vec![],
            },
        }
    }
}

/// What a targeting restriction looks like.
///
/// # Implementation Status
/// - No annotation = targets_for_spec() handles it and rules resolve against it
/// - `// UNIMPLEMENTED` = declared for card definitions; not yet handled in targeting logic
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

    // ------------------------------------------------------------------
    // Additional target specs — UNIMPLEMENTED
    // ------------------------------------------------------------------

    /// Target any land permanent.                                             // UNIMPLEMENTED
    AnyLand,
    /// Target any artifact permanent.                                         // UNIMPLEMENTED
    AnyArtifact,
    /// Target any enchantment permanent.                                      // UNIMPLEMENTED
    AnyEnchantment,
    /// Target any artifact or enchantment.                                    // UNIMPLEMENTED
    AnyArtifactOrEnchantment,
    /// Target any creature you control.                                       // UNIMPLEMENTED
    AnyCreatureYouControl,
    /// Target any creature an opponent controls.                              // UNIMPLEMENTED
    AnyCreatureOpponentControls,
    /// Target any instant or sorcery card (usually from graveyard/hand).     // UNIMPLEMENTED
    AnyInstantOrSorcery,
    /// Target any non-token creature.                                         // UNIMPLEMENTED
    NonTokenCreature,
    /// Target any noncreature permanent.                                      // UNIMPLEMENTED
    AnyNoncreaturePermanent,
    /// Target any planeswalker permanent.                                     // UNIMPLEMENTED
    AnyPlaneswalker,
    /// Target a creature with power N or less (N stored alongside in effect). // UNIMPLEMENTED
    CreatureWithPowerAtMost(i32),
    /// Target a creature with toughness N or less.                            // UNIMPLEMENTED
    CreatureWithToughnessAtMost(i32),
    /// Target a creature or land (e.g., Boseiju, Who Endures).               // UNIMPLEMENTED
    CreatureOrLand,
    /// Target a spell with a specific mana value or less.                     // UNIMPLEMENTED
    SpellWithManaValueAtMost(u32),
    /// Each opponent (no individual targeting — similar to NoTarget but
    /// explicitly multi-opponent for trigger clarity).                        // UNIMPLEMENTED
    EachOpponent,
    /// Each player including you.                                             // UNIMPLEMENTED
    EachPlayer,
    /// Each artifact (boardwipe style).                                       // UNIMPLEMENTED
    EachArtifact,
    /// Each enchantment.                                                      // UNIMPLEMENTED
    EachEnchantment,
    /// Each nonland permanent (boardwipe style).                              // UNIMPLEMENTED
    EachNonlandPermanent,
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
