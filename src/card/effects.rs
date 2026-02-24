use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::{CardDef, CardInstance, CardType, KeywordAbility, ObjectId, Subtype, ZoneType};
use crate::mana::Color;

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
    /// Add mana where the amount is determined dynamically at runtime.
    AddDynamicMana {
        color: Color,
        count: DynamicValue,
    },
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
