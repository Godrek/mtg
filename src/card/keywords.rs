use serde::{Deserialize, Serialize};

/// Keyword abilities that affect game rules directly.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum KeywordAbility {
    // --- Existing keywords ---
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

    // --- Combat evasion ---
    /// Can't be blocked (e.g., Invisible Stalker, Triton Shorestalker).
    Unblockable,
    /// Can only block/be blocked by creatures with shadow (e.g., Soltari Monk).
    Shadow,
    /// Can only block/be blocked by creatures with horsemanship (Portal Three Kingdoms).
    Horsemanship,
    /// Can't be blocked by creatures with greater power (e.g., Tetsuko Umezawa).
    Skulk,

    // --- Damage modification ---
    /// Damage dealt to creatures is dealt in the form of -1/-1 counters (e.g., Kulrath Knight).
    Wither,
    /// Damage to players as poison counters, to creatures as -1/-1 counters (e.g., Blighted Agent).
    Infect,
    /// When deals combat damage to player, that player gets N poison counters.
    /// The N is stored separately on the card definition; this flag enables the mechanic.
    Toxic,

    // --- Triggered keyword abilities ---
    /// When dies with no +1/+1 counters, return with a +1/+1 counter (e.g., Geralf's Messenger).
    Undying,
    /// When dies with no -1/-1 counters, return with a -1/-1 counter (e.g., Kitchen Finks).
    Persist,
    /// Whenever you cast a noncreature spell, this creature gets +1/+1 until EOT.
    Prowess,

    // --- Type modification ---
    /// This creature is every creature type (e.g., Morophon, Maskwood Nexus).
    Changeling,

    // --- Other ---
    /// Flanking: when blocked by a creature without flanking, blocker gets -1/-1 until EOT.
    Flanking,
    /// Ward: opponent must pay an additional cost to target this (simplified: counter unless pays {1}).
    Ward,
}
