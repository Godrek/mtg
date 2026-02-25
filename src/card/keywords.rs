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
    /// On cast, exile cards from top until you hit a nonland with lesser CMC; may cast for free.
    Cascade,
    /// On cast, copy this spell for each spell cast before it this turn.
    Storm,

    // --- Type modification ---
    /// This creature is every creature type (e.g., Morophon, Maskwood Nexus).
    Changeling,

    // --- Damage prevention ---
    /// Flanking: when blocked by a creature without flanking, blocker gets -1/-1 until EOT.
    Flanking,
    /// Ward: opponent must pay an additional cost to target this (simplified: counter unless pays {1}).
    Ward,

    // --- Cost modification keywords ---
    /// Convoke: tap creatures to help pay for this spell ({1} per tapped creature).
    Convoke,
    /// Delve: exile cards from your graveyard to help pay for this spell ({1} per exiled card).
    Delve,
    /// Affinity for artifacts: costs {1} less for each artifact you control.
    AffinityForArtifacts,

    // --- Alternative cost keywords (use the cost fields on CardDef) ---
    /// Flashback: may cast this from your graveyard for its flashback cost, then exile.
    Flashback,
    /// Kicker: may pay an additional cost for an enhanced effect.
    Kicker,
    /// Overload: may cast for overload cost; replaces "target" with "each".
    Overload,
    /// Escape: cast from graveyard by paying mana + exiling N cards from graveyard.
    Escape,
    /// Evoke: cast for evoke cost, sacrifice when ETB.
    Evoke,

    // --- Commander-specific ---
    /// Partner: this creature can be paired with another Partner creature as co-commanders.
    Partner,

    // --- Keyword actions (used as flags for triggered effects) ---
    /// Annihilator N: defending player sacrifices N permanents when this attacks.
    /// The N value is stored on the card definition.
    Annihilator,
    /// Exalted: whenever a creature you control attacks alone, it gets +1/+1 until EOT.
    Exalted,
    /// Extort: whenever you cast a spell, you may pay {W/B}. If you do, each opponent
    /// loses 1 life and you gain that much life.
    Extort,
}
