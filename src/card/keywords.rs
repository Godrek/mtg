use serde::{Deserialize, Serialize};

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
