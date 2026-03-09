use serde::{Deserialize, Serialize};

/// Keyword abilities that affect game rules directly (CR 702).
///
/// # Implementation Status Legend
/// - No annotation = fully implemented in rules engine
/// - `// PARTIAL: <note>` = declared; some rules handling exists but incomplete
/// - `// UNIMPLEMENTED` = declared for card definitions; no rules handling yet
///
/// Abilities that require a numeric parameter (e.g., Annihilator N, Rampage N)
/// store that value separately on the `CardDef` (e.g., `annihilator_value`); the
/// keyword flag here simply marks the mechanic as present.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum KeywordAbility {
    // =========================================================================
    // CR 702.2 — Deathtouch
    // =========================================================================
    /// Any amount of damage this deals to a creature is enough to destroy it.
    Deathtouch,

    // =========================================================================
    // CR 702.3 — Defender
    // =========================================================================
    /// This creature can't attack.
    Defender,

    // =========================================================================
    // CR 702.4 — Double Strike
    // =========================================================================
    /// This creature deals both first-strike and regular combat damage.
    DoubleStrike,

    // =========================================================================
    // CR 702.6 — Equip  (handled as ActivatedAbility; no enum variant needed)
    // =========================================================================

    // =========================================================================
    // CR 702.7 — First Strike
    // =========================================================================
    /// This creature deals combat damage before creatures without first strike.
    FirstStrike,

    // =========================================================================
    // CR 702.8 — Flash
    // =========================================================================
    /// You may cast this spell any time you could cast an instant.
    Flash,

    // =========================================================================
    // CR 702.9 — Flying
    // =========================================================================
    /// This creature can only be blocked by creatures with flying or reach.
    Flying,

    // =========================================================================
    // CR 702.10 — Haste
    // =========================================================================
    /// This creature can attack and tap immediately when it enters.
    Haste,

    // =========================================================================
    // CR 702.11 — Hexproof
    // =========================================================================
    /// This permanent can't be the target of spells or abilities opponents control.
    Hexproof,

    // =========================================================================
    // CR 702.12 — Indestructible
    // =========================================================================
    /// Effects that say "destroy" don't destroy this permanent. Lethal damage
    /// doesn't destroy it either.
    Indestructible,

    // =========================================================================
    // CR 702.13 — Intimidate (obsolete, replaced by Menace in most contexts)
    // =========================================================================
    /// This creature can only be blocked by artifact creatures and/or creatures
    /// that share a color with it.
    Intimidate,

    // =========================================================================
    // CR 702.14 — Landwalk (cycle of obsolete evasion keywords)
    // =========================================================================
    /// Can't be blocked if defending player controls a Forest.                // UNIMPLEMENTED
    Forestwalk,
    /// Can't be blocked if defending player controls an Island.               // UNIMPLEMENTED
    Islandwalk,
    /// Can't be blocked if defending player controls a Mountain.              // UNIMPLEMENTED
    Mountainwalk,
    /// Can't be blocked if defending player controls a Plains.                // UNIMPLEMENTED
    Plainswalk,
    /// Can't be blocked if defending player controls a Swamp.                 // UNIMPLEMENTED
    Swampwalk,
    /// Can't be blocked if defending player controls a land of the named type.// UNIMPLEMENTED
    Landwalk,

    // =========================================================================
    // CR 702.15 — Lifelink
    // =========================================================================
    /// Damage dealt by this creature also causes its controller to gain that
    /// much life.
    Lifelink,

    // =========================================================================
    // CR 702.16 — Protection
    // =========================================================================
    // PARTIAL: simplified — full protection needs a qualifier (color, type, etc.)
    /// Protection from [quality]: can't be Damaged, Enchanted/Equipped,
    /// Blocked, or Targeted by sources of that quality.
    Protection,
    /// Protection from everything (e.g., Progenitus, True-Name Nemesis).     // PARTIAL
    ProtectionFromEverything,

    // =========================================================================
    // CR 702.17 — Reach
    // =========================================================================
    /// This creature can block creatures with flying.
    Reach,

    // =========================================================================
    // CR 702.18 — Shroud
    // =========================================================================
    /// This permanent can't be the target of spells or abilities (by anyone).
    Shroud,

    // =========================================================================
    // CR 702.19 — Trample
    // =========================================================================
    /// If this creature would deal more combat damage than needed to destroy all
    /// blockers, the excess is dealt to the defending player or planeswalker.
    Trample,

    // =========================================================================
    // CR 702.20 — Vigilance
    // =========================================================================
    /// Attacking doesn't cause this creature to tap.
    Vigilance,

    // =========================================================================
    // CR 702.21 — Banding (obsolete)
    // =========================================================================
    /// Creatures with banding can form bands to attack or block together.     // UNIMPLEMENTED
    Banding,

    // =========================================================================
    // CR 702.22 — Rampage N
    // =========================================================================
    /// Whenever this creature becomes blocked, it gets +N/+N until EOT for
    /// each creature blocking it beyond the first. N stored on CardDef.       // UNIMPLEMENTED
    Rampage,

    // =========================================================================
    // CR 702.23 — Cumulative Upkeep
    // =========================================================================
    /// At the beginning of your upkeep, put an age counter on this permanent,
    /// then sacrifice it unless you pay its upkeep cost times its age counters.// UNIMPLEMENTED
    CumulativeUpkeep,

    // =========================================================================
    // CR 702.24 — Flanking
    // =========================================================================
    // PARTIAL: trigger declared; -1/-1 application incomplete.
    /// Whenever this creature becomes blocked by a creature without flanking,
    /// the blocking creature gets -1/-1 until EOT.
    Flanking,

    // =========================================================================
    // CR 702.25 — Phasing
    // =========================================================================
    /// At the beginning of each of your untap steps, this phases in or out.  // UNIMPLEMENTED
    Phasing,

    // =========================================================================
    // CR 702.26 — Buyback
    // =========================================================================
    /// You may pay an additional cost when casting this spell. If you do,
    /// put it into your hand instead of your graveyard as it resolves.        // UNIMPLEMENTED
    Buyback,

    // =========================================================================
    // CR 702.27 — Shadow
    // =========================================================================
    /// This creature can only block or be blocked by creatures with shadow.
    Shadow,

    // =========================================================================
    // CR 702.28 — Cycling (cost)
    // =========================================================================
    /// Discard this card, pay cost: draw a card.                              // UNIMPLEMENTED
    Cycling,
    /// Typecycling variants (Forestcycling, etc.).                            // UNIMPLEPLEMENTED
    Typecycling,

    // =========================================================================
    // CR 702.29 — Echo
    // =========================================================================
    /// At the beginning of your upkeep, if this came under your control since
    /// the beginning of your last upkeep, sacrifice it unless you pay its echo
    /// cost.                                                                  // UNIMPLEMENTED
    Echo,

    // =========================================================================
    // CR 702.30 — Horsemanship
    // =========================================================================
    /// Like flying, but only creatures with horsemanship can block.
    Horsemanship,

    // =========================================================================
    // CR 702.31 — Fading N
    // =========================================================================
    /// Enters with N fade counters. At upkeep, remove one. If you can't,
    /// sacrifice it.                                                          // UNIMPLEMENTED
    Fading,

    // =========================================================================
    // CR 702.32 — Kicker
    // =========================================================================
    // PARTIAL: cost parsing present; effect branching simplified.
    /// You may pay an additional kicker cost as you cast this spell for an
    /// enhanced effect.
    Kicker,
    /// Multikicker: like kicker but may be paid multiple times.               // UNIMPLEMENTED
    Multikicker,

    // =========================================================================
    // CR 702.33 — Flashback (cost)
    // =========================================================================
    // PARTIAL: graveyard casting present; exile-after-resolution sometimes missed.
    /// You may cast this card from your graveyard for its flashback cost, then
    /// exile it.
    Flashback,

    // =========================================================================
    // CR 702.34 — Madness (cost)
    // =========================================================================
    /// If you discard this card, discard it into exile. When you do, cast it
    /// for its madness cost or put it into your graveyard.                    // UNIMPLEMENTED
    Madness,

    // =========================================================================
    // CR 702.35 — Fear (obsolete, replaced by Intimidate)
    // =========================================================================
    /// This creature can only be blocked by artifact creatures and black
    /// creatures.
    Fear,

    // =========================================================================
    // CR 702.36 — Morph (cost)
    // =========================================================================
    /// You may cast this card face down as a 2/2 creature for {3}. Turn it
    /// face up any time for its morph cost.                                   // UNIMPLEMENTED
    Morph,
    /// Megamorph: like morph, but also puts a +1/+1 counter on it when flipped.// UNIMPLEMENTED
    Megamorph,

    // =========================================================================
    // CR 702.37 — Amplify N
    // =========================================================================
    /// As this enters, put N +1/+1 counters on it for each card you reveal
    /// in your hand that shares a creature type with it.                      // UNIMPLEMENTED
    Amplify,

    // =========================================================================
    // CR 702.38 — Provoke
    // =========================================================================
    /// When this creature attacks, you may have target creature the defending
    /// player controls untap and block it if able.                            // UNIMPLEMENTED
    Provoke,

    // =========================================================================
    // CR 702.39 — Storm
    // =========================================================================
    // PARTIAL: copy count computed; copies not always placed on stack.
    /// When you cast this spell, copy it for each other spell cast before it
    /// this turn.
    Storm,

    // =========================================================================
    // CR 702.40 — Affinity (for [quality])
    // =========================================================================
    // PARTIAL: only affinity for artifacts implemented.
    /// This spell costs {1} less to cast for each artifact you control.
    AffinityForArtifacts,
    /// Generic affinity for a card type (Artifacts, Swamps, etc.).            // UNIMPLEMENTED
    Affinity,

    // =========================================================================
    // CR 702.41 — Entwine (cost)
    // =========================================================================
    /// If you pay the entwine cost, choose all modes of this modal spell.     // UNIMPLEMENTED
    Entwine,

    // =========================================================================
    // CR 702.42 — Modular N
    // =========================================================================
    /// Enters with N +1/+1 counters. When it dies, move those counters to a
    /// target artifact creature.                                              // UNIMPLEMENTED
    Modular,

    // =========================================================================
    // CR 702.43 — Sunburst
    // =========================================================================
    /// Enters with a counter for each color of mana spent to cast it.        // UNIMPLEMENTED
    Sunburst,

    // =========================================================================
    // CR 702.44 — Bushido N
    // =========================================================================
    /// Whenever this creature blocks or becomes blocked, it gets +N/+N until
    /// EOT.                                                                   // UNIMPLEMENTED
    Bushido,

    // =========================================================================
    // CR 702.45 — Soulshift N
    // =========================================================================
    /// When this creature dies, return target Spirit card with mana value N or
    /// less from your graveyard to your hand.                                 // UNIMPLEMENTED
    Soulshift,

    // =========================================================================
    // CR 702.46 — Splice (onto Arcane, cost)
    // =========================================================================
    /// Reveal this card from your hand and pay its splice cost any time you
    /// cast an Arcane spell to add this card's text to that spell.            // UNIMPLEMENTED
    Splice,

    // =========================================================================
    // CR 702.47 — Offering (subtype)
    // =========================================================================
    /// You may cast this card any time you could cast an instant by sacrificing
    /// a permanent of the named subtype and paying the cost difference.       // UNIMPLEMENTED
    Offering,

    // =========================================================================
    // CR 702.48 — Ninjutsu (cost)
    // =========================================================================
    /// Return an unblocked attacker to hand: put this from hand onto the
    /// battlefield tapped and attacking.                                      // UNIMPLEMENTED
    Ninjutsu,
    /// Commander Ninjutsu variant.                                            // UNIMPLEMENTED
    CommanderNinjutsu,

    // =========================================================================
    // CR 702.49 — Epic
    // =========================================================================
    /// For the rest of the game you can't cast spells. At the beginning of
    /// each of your upkeeps, copy this spell.                                 // UNIMPLEMENTED
    Epic,

    // =========================================================================
    // CR 702.50 — Convoke
    // =========================================================================
    // PARTIAL: cost reduction mechanic present; tapping creatures simplified.
    /// Your creatures can help cast this spell. Tap a creature to pay {1} of
    /// this spell's cost (or a colored mana of that creature's color).
    Convoke,

    // =========================================================================
    // CR 702.52 — Dredge N
    // =========================================================================
    /// If you would draw a card, instead you may mill N cards and return this
    /// from your graveyard to your hand.                                      // UNIMPLEMENTED
    Dredge,

    // =========================================================================
    // CR 702.53 — Transmute (cost)
    // =========================================================================
    /// Discard this card, pay cost: search your library for a card with the
    /// same mana value.                                                       // UNIMPLEMENTED
    Transmute,

    // =========================================================================
    // CR 702.54 — Bloodthirst N
    // =========================================================================
    /// If an opponent was dealt damage this turn, this enters with N +1/+1
    /// counters.                                                              // UNIMPLEMENTED
    Bloodthirst,

    // =========================================================================
    // CR 702.55 — Haunt
    // =========================================================================
    /// When this creature dies, exile it haunting another creature. When the
    /// haunted creature dies, trigger this card's haunting ability.           // UNIMPLEMENTED
    Haunt,

    // =========================================================================
    // CR 702.56 — Replicate (cost)
    // =========================================================================
    /// When you cast this spell, copy it for each time you paid its replicate
    /// cost.                                                                  // UNIMPLEMENTED
    Replicate,

    // =========================================================================
    // CR 702.57 — Forecast (cost)
    // =========================================================================
    /// Activated ability usable only from your hand during your upkeep.      // UNIMPLEMENTED
    Forecast,

    // =========================================================================
    // CR 702.58 — Graft N
    // =========================================================================
    /// Enters with N +1/+1 counters. When another creature enters, you may
    /// move a counter from this to it.                                        // UNIMPLEMENTED
    Graft,

    // =========================================================================
    // CR 702.59 — Recover (cost)
    // =========================================================================
    /// When a creature is put into any graveyard, you may pay the recover cost
    /// to return this card from your graveyard to your hand.                  // UNIMPLEMENTED
    Recover,

    // =========================================================================
    // CR 702.60 — Ripple N
    // =========================================================================
    /// When you cast this spell, reveal the top N cards of your library. You
    /// may cast cards with the same name without paying their mana costs.     // UNIMPLEMENTED
    Ripple,

    // =========================================================================
    // CR 702.61 — Split Second
    // =========================================================================
    /// While this spell is on the stack, players can't cast spells or activate
    /// abilities that aren't mana abilities.                                  // UNIMPLEMENTED
    SplitSecond,

    // =========================================================================
    // CR 702.62 — Suspend N (cost)
    // =========================================================================
    /// Rather than cast this, pay cost and exile it with N time counters.
    /// At each upkeep remove a counter; when last is removed, cast for free.  // UNIMPLEMENTED
    Suspend,

    // =========================================================================
    // CR 702.63 — Vanishing N
    // =========================================================================
    /// Enters with N time counters. At upkeep remove a counter. When you
    /// can't, sacrifice it.                                                   // UNIMPLEMENTED
    Vanishing,

    // =========================================================================
    // CR 702.64 — Absorb N
    // =========================================================================
    /// If a source would deal damage to this creature, prevent N of that
    /// damage.                                                                // UNIMPLEMENTED
    Absorb,

    // =========================================================================
    // CR 702.65 — Aura Swap (cost)
    // =========================================================================
    /// Tap, pay cost: exchange this Aura with an Aura card in your hand.     // UNIMPLEMENTED
    AuraSwap,

    // =========================================================================
    // CR 702.66 — Delve
    // =========================================================================
    // PARTIAL: cost reduction counted; graveyard exile not always enforced.
    /// Exile cards from your graveyard to pay for each {1} in this spell's
    /// mana cost.
    Delve,

    // =========================================================================
    // CR 702.67 — Fortify (cost)
    // =========================================================================
    /// Tap, pay cost: attach this Fortification to target land you control.
    /// Fortify only as a sorcery.                                             // UNIMPLEMENTED
    Fortify,

    // =========================================================================
    // CR 702.68 — Frenzy N
    // =========================================================================
    /// Whenever this creature attacks and isn't blocked, it gets +N/+0
    /// until EOT.                                                             // UNIMPLEMENTED
    Frenzy,

    // =========================================================================
    // CR 702.69 — Gravestorm
    // =========================================================================
    /// When you cast this spell, copy it for each permanent put into a
    /// graveyard this turn.                                                   // UNIMPLEMENTED
    Gravestorm,

    // =========================================================================
    // CR 702.70 — Poisonous N
    // =========================================================================
    /// Whenever this creature deals combat damage to a player, that player
    /// gets N poison counters. (10 poison counters = that player loses.)      // UNIMPLEMENTED
    Poisonous,

    // =========================================================================
    // CR 702.71 — Transfigure (cost)
    // =========================================================================
    /// Pay, Tap, Sacrifice this creature: search for a creature card with the
    /// same mana value and put it onto the battlefield.                       // UNIMPLEMENTED
    Transfigure,

    // =========================================================================
    // CR 702.72 — Champion (subtype)
    // =========================================================================
    /// When this enters, sacrifice it unless you exile another [subtype] you
    /// control. When this leaves, return the exiled card.                     // UNIMPLEMENTED
    Champion,

    // =========================================================================
    // CR 702.73 — Changeling
    // =========================================================================
    /// This card is every creature type.
    Changeling,

    // =========================================================================
    // CR 702.74 — Evoke (cost)
    // =========================================================================
    // PARTIAL: evoke cost listed; sacrifice-on-ETB trigger incomplete.
    /// You may cast this for its evoke cost. If you do, its enters-the-
    /// battlefield ability triggers, then you sacrifice it.
    Evoke,

    // =========================================================================
    // CR 702.75 — Hideaway N
    // =========================================================================
    /// When this land enters, look at the top N cards of your library, exile
    /// one face down, put the rest on the bottom.                             // UNIMPLEMENTED
    Hideaway,

    // =========================================================================
    // CR 702.76 — Prowl (cost)
    // =========================================================================
    /// You may cast this for its prowl cost if you dealt combat damage to a
    /// player this turn with a creature of the appropriate type.              // UNIMPLEMENTED
    Prowl,

    // =========================================================================
    // CR 702.77 — Reinforce N (cost)
    // =========================================================================
    /// Discard this card, pay cost: put N +1/+1 counters on target creature. // UNIMPLEMENTED
    Reinforce,

    // =========================================================================
    // CR 702.78 — Conspire
    // =========================================================================
    /// As you cast this spell, tap two untapped creatures sharing a color with
    /// it to copy it.                                                         // UNIMPLEMENTED
    Conspire,

    // =========================================================================
    // CR 702.79 — Persist
    // =========================================================================
    /// When this creature dies with no -1/-1 counters, return it with a
    /// -1/-1 counter.
    Persist,

    // =========================================================================
    // CR 702.80 — Wither
    // =========================================================================
    // PARTIAL: damage dealt as -1/-1 counters; counter removal on cleanup incomplete.
    /// Damage dealt by this source is dealt in the form of -1/-1 counters
    /// instead of normal damage.
    Wither,

    // =========================================================================
    // CR 702.81 — Retrace
    // =========================================================================
    /// You may cast this card from your graveyard by discarding a land in
    /// addition to paying its other costs.                                    // UNIMPLEMENTED
    Retrace,

    // =========================================================================
    // CR 702.82 — Devour N
    // =========================================================================
    /// As this creature enters, you may sacrifice any number of creatures. It
    /// enters with N +1/+1 counters for each creature sacrificed this way.   // UNIMPLEMENTED
    Devour,

    // =========================================================================
    // CR 702.83 — Exalted
    // =========================================================================
    /// Whenever a creature you control attacks alone, it gets +1/+1 until EOT.
    Exalted,

    // =========================================================================
    // CR 702.84 — Unearth (cost)
    // =========================================================================
    /// Exile this from your graveyard: return it to the battlefield with haste.
    /// Exile it at EOT or if it would leave the battlefield.                  // UNIMPLEMENTED
    Unearth,

    // =========================================================================
    // CR 702.85 — Cascade
    // =========================================================================
    // PARTIAL: exile loop present; free cast sometimes skipped.
    /// When you cast this spell, exile cards from the top of your library until
    /// you exile a nonland card with lesser mana value; you may cast it for free.
    Cascade,

    // =========================================================================
    // CR 702.86 — Annihilator N
    // =========================================================================
    /// Whenever this creature attacks, defending player sacrifices N permanents.
    /// N is stored on the CardDef.
    Annihilator,

    // =========================================================================
    // CR 702.87 — Level Up (cost)
    // =========================================================================
    /// Sorcery-speed activated ability that puts level counters on this
    /// creature; grants increased P/T and abilities at each level threshold.  // UNIMPLEMENTED
    LevelUp,

    // =========================================================================
    // CR 702.88 — Rebound
    // =========================================================================
    /// If you cast this from your hand, exile it as it resolves. At the
    /// beginning of your next upkeep, you may cast it for free.               // UNIMPLEMENTED
    Rebound,

    // =========================================================================
    // CR 702.89 — Totem Armor
    // =========================================================================
    /// If enchanted permanent would be destroyed, instead remove all damage
    /// from it and destroy this Aura.                                         // UNIMPLEMENTED
    TotemArmor,

    // =========================================================================
    // CR 702.90 — Infect
    // =========================================================================
    // PARTIAL: damage routed to -1/-1 counters on creatures / poison on players.
    /// Damage this deals to creatures is -1/-1 counters; to players is poison
    /// counters. (10 poison counters = that player loses.)
    Infect,

    // =========================================================================
    // CR 702.91 — Battle Cry
    // =========================================================================
    /// Whenever this creature attacks, each other attacking creature gets +1/+0
    /// until EOT.                                                             // UNIMPLEMENTED
    BattleCry,

    // =========================================================================
    // CR 702.92 — Living Weapon
    // =========================================================================
    /// When this Equipment enters, create a 0/0 black Phyrexian Germ creature
    /// token, then attach this to it.                                         // UNIMPLEMENTED
    LivingWeapon,

    // =========================================================================
    // CR 702.93 — Undying
    // =========================================================================
    /// When this creature dies with no +1/+1 counters, return it with a
    /// +1/+1 counter.
    Undying,

    // =========================================================================
    // CR 702.94 — Miracle (cost)
    // =========================================================================
    /// You may cast this card for its miracle cost when you draw it, if it's
    /// the first card drawn this turn.                                        // UNIMPLEMENTED
    Miracle,

    // =========================================================================
    // CR 702.95 — Soulbond
    // =========================================================================
    /// You may pair this creature with another unpaired creature when either
    /// enters. They remain paired while you control both; share bonuses.      // UNIMPLEMENTED
    Soulbond,

    // =========================================================================
    // CR 702.96 — Overload (cost)
    // =========================================================================
    // PARTIAL: overload cost present; "target" → "each" substitution incomplete.
    /// If you cast this for its overload cost, replace "target" with "each."
    Overload,

    // =========================================================================
    // CR 702.97 — Scavenge (cost)
    // =========================================================================
    /// Exile this from your graveyard, pay cost: put +1/+1 counters on target
    /// creature equal to this card's power. Scavenge only as a sorcery.      // UNIMPLEMENTED
    Scavenge,

    // =========================================================================
    // CR 702.98 — Unleash
    // =========================================================================
    /// You may have this creature enter with a +1/+1 counter. If it has a
    /// +1/+1 counter, it can't block.                                        // UNIMPLEMENTED
    Unleash,

    // =========================================================================
    // CR 702.99 — Cipher
    // =========================================================================
    /// Then you may exile this spell card encoded on a creature you control.
    /// Whenever that creature deals combat damage to a player, its controller
    /// may cast a copy of the encoded card.                                   // UNIMPLEMENTED
    Cipher,

    // =========================================================================
    // CR 702.100 — Evolve
    // =========================================================================
    /// Whenever a creature enters under your control, if it has greater power
    /// or toughness than this creature, put a +1/+1 counter on this creature. // UNIMPLEMENTED
    Evolve,

    // =========================================================================
    // CR 702.101 — Extort
    // =========================================================================
    // PARTIAL: trigger fired; W/B hybrid cost payment not enforced.
    /// Whenever you cast a spell, you may pay {W/B}. If you do, each opponent
    /// loses 1 life and you gain that much life.
    Extort,

    // =========================================================================
    // CR 702.102 — Fuse
    // =========================================================================
    /// You may cast both halves of this split card (paying both costs).       // UNIMPLEMENTED
    Fuse,

    // =========================================================================
    // CR 702.103 — Bestow (cost)
    // =========================================================================
    /// If cast for its bestow cost, this is an Aura spell that becomes a
    /// creature again if not attached to a creature.                          // UNIMPLEMENTED
    Bestow,

    // =========================================================================
    // CR 702.104 — Tribute N
    // =========================================================================
    /// As this enters, an opponent may put N +1/+1 counters on it. If they
    /// don't, you get an additional effect.                                   // UNIMPLEMENTED
    Tribute,

    // =========================================================================
    // CR 702.105 — Dethrone
    // =========================================================================
    /// Whenever this creature attacks the player with the most life (or tied),
    /// put a +1/+1 counter on this creature.                                 // UNIMPLEMENTED
    Dethrone,

    // =========================================================================
    // CR 702.106 — Hidden Agenda
    // =========================================================================
    /// Start the game with this conspiracy face down. Turn it face up any time
    /// and secretly name a card. [effect when the named card is cast]         // UNIMPLEMENTED
    HiddenAgenda,

    // =========================================================================
    // CR 702.107 — Outlast (cost)
    // =========================================================================
    /// Tap, pay cost: put a +1/+1 counter on this creature. Activate only as
    /// a sorcery.                                                             // UNIMPLEMENTED
    Outlast,

    // =========================================================================
    // CR 702.108 — Prowess
    // =========================================================================
    // PARTIAL: +1/+1 trigger fires on noncreature spells; cleanup at EOT simplified.
    /// Whenever you cast a noncreature spell, this creature gets +1/+1
    /// until EOT.
    Prowess,

    // =========================================================================
    // CR 702.109 — Dash (cost)
    // =========================================================================
    /// You may cast this for its dash cost. If you do, it gains haste and is
    /// returned to your hand at the beginning of the next end step.           // UNIMPLEMENTED
    Dash,

    // =========================================================================
    // CR 702.110 — Exploit
    // =========================================================================
    /// When this creature enters, you may sacrifice a creature. When you do,
    /// this creature's exploit ability triggers.                              // UNIMPLEMENTED
    Exploit,

    // =========================================================================
    // CR 702.111 — Menace
    // =========================================================================
    /// This creature can't be blocked except by two or more creatures.
    Menace,

    // =========================================================================
    // CR 702.112 — Renown N
    // =========================================================================
    /// When this creature deals combat damage to a player, if it isn't
    /// renowned, put N +1/+1 counters on it and it becomes renowned.         // UNIMPLEMENTED
    Renown,

    // =========================================================================
    // CR 702.113 — Awaken N (cost)
    // =========================================================================
    /// If cast for its awaken cost, also put N +1/+1 counters on target land
    /// you control and it becomes a 0/0 Elemental creature in addition to its
    /// other types.                                                           // UNIMPLEMENTED
    Awaken,

    // =========================================================================
    // CR 702.114 — Devoid
    // =========================================================================
    /// This card has no color.                                                // UNIMPLEMENTED
    Devoid,

    // =========================================================================
    // CR 702.115 — Ingest
    // =========================================================================
    /// Whenever this creature deals combat damage to a player, that player
    /// exiles the top card of their library.                                  // UNIMPLEMENTED
    Ingest,

    // =========================================================================
    // CR 702.116 — Myriad
    // =========================================================================
    /// Whenever this creature attacks, for each opponent other than defending
    /// player, create a tapped-and-attacking token copy of it vs that player.
    /// Exile the tokens at end of combat.                                     // UNIMPLEMENTED
    Myriad,

    // =========================================================================
    // CR 702.117 — Surge (cost)
    // =========================================================================
    /// You may cast this for its surge cost if you or a teammate cast another
    /// spell this turn.                                                       // UNIMPLEMENTED
    Surge,

    // =========================================================================
    // CR 702.118 — Skulk
    // =========================================================================
    /// This creature can't be blocked by creatures with greater power.
    Skulk,

    // =========================================================================
    // CR 702.119 — Emerge (cost)
    // =========================================================================
    /// You may cast this by sacrificing a creature and paying the emerge cost
    /// reduced by that creature's mana value.                                 // UNIMPLEMENTED
    Emerge,

    // =========================================================================
    // CR 702.120 — Escalate (cost)
    // =========================================================================
    /// Pay this cost for each mode chosen beyond the first when casting a
    /// modal spell.                                                           // UNIMPLEMENTED
    Escalate,

    // =========================================================================
    // CR 702.121 — Melee
    // =========================================================================
    /// Whenever this creature attacks, it gets +1/+1 until EOT for each
    /// opponent you attacked this turn.                                       // UNIMPLEMENTED
    Melee,

    // =========================================================================
    // CR 702.122 — Crew N
    // =========================================================================
    /// Tap any number of creatures you control with total power N or more:
    /// This Vehicle becomes an artifact creature until EOT.                   // UNIMPLEMENTED
    Crew,

    // =========================================================================
    // CR 702.123 — Fabricate N
    // =========================================================================
    /// When this creature enters, put N +1/+1 counters on it OR create N
    /// 1/1 colorless Servo artifact creature tokens.                          // UNIMPLEMENTED
    Fabricate,

    // =========================================================================
    // CR 702.124 — Partner (and Partner with [name])
    // =========================================================================
    // PARTIAL: pairing allowed; full partner abilities not validated.
    /// This creature can be paired with another Partner creature as a
    /// co-commander.
    Partner,
    /// Partner with [name]: can only partner with the specific named card.    // UNIMPLEMENTED
    PartnerWith,
    /// Friends forever: like Partner, for Stranger Things crossover cards.    // UNIMPLEMENTED
    FriendsForever,
    /// Doctor's companion: pairs with a Doctor creature.                      // UNIMPLEMENTED
    DoctorsCompanion,

    // =========================================================================
    // CR 702.125 — Undaunted
    // =========================================================================
    /// This spell costs {1} less to cast for each opponent you have.         // UNIMPLEMENTED
    Undaunted,

    // =========================================================================
    // CR 702.126 — Improvise
    // =========================================================================
    /// Your artifacts can help cast this spell. Each artifact you tap after
    /// activating mana abilities pays for {1}.                                // UNIMPLEMENTED
    Improvise,

    // =========================================================================
    // CR 702.127 — Aftermath
    // =========================================================================
    /// Cast this half only from your graveyard. Exile it when it resolves or
    /// would be put into a graveyard from anywhere.                           // UNIMPLEMENTED
    Aftermath,

    // =========================================================================
    // CR 702.128 — Embalm (cost)
    // =========================================================================
    /// Pay cost, Exile this from your graveyard: Create a token that's a copy
    /// of it except it's a white Zombie in addition to its other types.       // UNIMPLEMENTED
    Embalm,

    // =========================================================================
    // CR 702.129 — Eternalize (cost)
    // =========================================================================
    /// Pay cost, Exile this from your graveyard: Create a 4/4 black Zombie
    /// token copy of it.                                                      // UNIMPLEMENTED
    Eternalize,

    // =========================================================================
    // CR 702.130 — Afflict N
    // =========================================================================
    /// Whenever this creature becomes blocked, defending player loses N life. // UNIMPLEMENTED
    Afflict,

    // =========================================================================
    // CR 702.131 — Ascend
    // =========================================================================
    /// If you control ten or more permanents, you get the city's blessing for
    /// the rest of the game.                                                  // UNIMPLEMENTED
    Ascend,

    // =========================================================================
    // CR 702.132 — Assist
    // =========================================================================
    /// Another player can pay up to this spell's generic mana cost for you.  // UNIMPLEMENTED
    Assist,

    // =========================================================================
    // CR 702.133 — Jump-Start
    // =========================================================================
    /// You may cast this from your graveyard by discarding a card in addition
    /// to paying its other costs. Then exile this card.                       // UNIMPLEMENTED
    JumpStart,

    // =========================================================================
    // CR 702.134 — Mentor
    // =========================================================================
    /// Whenever this creature attacks, put a +1/+1 counter on target attacking
    /// creature with lesser power.                                            // UNIMPLEMENTED
    Mentor,

    // =========================================================================
    // CR 702.135 — Afterlife N
    // =========================================================================
    /// When this creature dies, create N 1/1 white and black Spirit creature
    /// tokens with flying.                                                    // UNIMPLEMENTED
    Afterlife,

    // =========================================================================
    // CR 702.136 — Riot
    // =========================================================================
    /// This creature enters with your choice of a +1/+1 counter or haste.   // UNIMPLEMENTED
    Riot,

    // =========================================================================
    // CR 702.137 — Spectacle (cost)
    // =========================================================================
    /// You may cast this for its spectacle cost if an opponent lost life this
    /// turn.                                                                  // UNIMPLEMENTED
    Spectacle,

    // =========================================================================
    // CR 702.138 — Escape (cost, N cards)
    // =========================================================================
    // PARTIAL: cast-from-graveyard path present; exile-N-cards cost not fully enforced.
    /// Cast this from your graveyard by paying the escape cost and exiling N
    /// other cards from your graveyard.
    Escape,

    // =========================================================================
    // CR 702.139 — Companion
    // =========================================================================
    /// If this card satisfies its companion condition, you may put it into your
    /// hand from outside the game for {3} once per game.                      // UNIMPLEMENTED
    Companion,

    // =========================================================================
    // CR 702.140 — Mutate (cost)
    // =========================================================================
    /// Cast this for its mutate cost and put it over or under target non-Human
    /// creature you own. The merged permanent has the characteristics of the
    /// top card plus all abilities from cards underneath.                     // UNIMPLEMENTED
    Mutate,

    // =========================================================================
    // CR 702.141 — Encore (cost)
    // =========================================================================
    /// Pay cost, Exile this from your graveyard: For each opponent, create a
    /// tapped-and-attacking token copy of it vs that opponent. Sacrifice them
    /// at beginning of next end step. Activate only as a sorcery.            // UNIMPLEMENTED
    Encore,

    // =========================================================================
    // CR 702.142 — Boast (cost)
    // =========================================================================
    /// Activated ability usable only once per turn and only if this creature
    /// attacked this turn.                                                    // UNIMPLEMENTED
    Boast,

    // =========================================================================
    // CR 702.143 — Foretell (cost)
    // =========================================================================
    /// During your turn, pay {2} and exile this face down. Cast it later for
    /// its foretell cost.                                                     // UNIMPLEMENTED
    Foretell,

    // =========================================================================
    // CR 702.144 — Demonstrate
    // =========================================================================
    /// When you cast this spell, you may copy it. If you do, choose an
    /// opponent who also copies it.                                           // UNIMPLEMENTED
    Demonstrate,

    // =========================================================================
    // CR 702.145 — Daybound / Nightbound
    // =========================================================================
    /// Daybound: if a player casts no spells during their own turn, it becomes
    /// night and this transforms into its nightbound face, and vice versa.    // UNIMPLEMENTED
    Daybound,
    /// Nightbound: the reverse face of a daybound card.                       // UNIMPLEMENTED
    Nightbound,

    // =========================================================================
    // CR 702.146 — Disturb (cost)
    // =========================================================================
    /// You may cast this card from your graveyard transformed for its disturb
    /// cost. If it would be put into a graveyard, exile it instead.           // UNIMPLEMENTED
    Disturb,

    // =========================================================================
    // CR 702.147 — Decayed
    // =========================================================================
    /// This creature can't block. When it attacks, sacrifice it at end of
    /// combat.                                                                // UNIMPLEMENTED
    Decayed,

    // =========================================================================
    // CR 702.148 — Cleave (cost)
    // =========================================================================
    /// You may cast this for its cleave cost. If you do, remove the words in
    /// square brackets from its rules text.                                   // UNIMPLEMENTED
    Cleave,

    // =========================================================================
    // CR 702.149 — Training
    // =========================================================================
    /// Whenever this creature attacks with another creature with greater power,
    /// put a +1/+1 counter on this creature.                                 // UNIMPLEMENTED
    Training,

    // =========================================================================
    // CR 702.150 — Compleated
    // =========================================================================
    /// If a Phyrexian mana symbol is paid with life, this planeswalker enters
    /// with two fewer loyalty counters.                                       // UNIMPLEMENTED
    Compleated,

    // =========================================================================
    // CR 702.151 — Reconfigure (cost)
    // =========================================================================
    /// Attach to target creature you control, or unattach. While attached,
    /// this isn't a creature. Reconfigure only as a sorcery.                  // UNIMPLEMENTED
    Reconfigure,

    // =========================================================================
    // CR 702.152 — Blitz (cost)
    // =========================================================================
    /// If cast for its blitz cost, it gains haste and "when this dies, draw a
    /// card." Sacrifice it at the beginning of the next end step.             // UNIMPLEMENTED
    Blitz,

    // =========================================================================
    // CR 702.153 — Casualty N
    // =========================================================================
    /// As an additional cost to cast this spell, you may sacrifice a creature
    /// with power N or greater. If you do, copy this spell.                   // UNIMPLEMENTED
    Casualty,

    // =========================================================================
    // CR 702.154 — Enlist
    // =========================================================================
    /// As this creature attacks, you may tap a nonattacking creature you
    /// control without summoning sickness. If you do, add its power to this
    /// creature's power until EOT.                                            // UNIMPLEMENTED
    Enlist,

    // =========================================================================
    // CR 702.155 — Read Ahead
    // =========================================================================
    /// Choose a chapter and start this Saga with that many lore counters.    // UNIMPLEMENTED
    ReadAhead,

    // =========================================================================
    // CR 702.156 — Ravenous
    // =========================================================================
    /// This creature enters with X +1/+1 counters on it. If X is 5 or more,
    /// draw a card.                                                           // UNIMPLEMENTED
    Ravenous,

    // =========================================================================
    // CR 702.157 — Squad (cost)
    // =========================================================================
    /// As an additional cost, pay this cost any number of times. When this
    /// creature enters, for each time you paid the squad cost, create a token
    /// copy of it.                                                            // UNIMPLEMENTED
    Squad,

    // =========================================================================
    // CR 702.158 — Prototype (cost, P/T)
    // =========================================================================
    /// You may cast this with a different mana cost, power, and toughness from
    /// its prototype alternative stats.                                       // UNIMPLEMENTED
    Prototype,

    // =========================================================================
    // CR 702.159 — Living Metal
    // =========================================================================
    /// As long as it's your turn, this Vehicle is also a creature.           // UNIMPLEMENTED
    LivingMetal,

    // =========================================================================
    // CR 702.160 — For Mirrodin!
    // =========================================================================
    /// When this Equipment enters, create a 2/2 red Rebel creature token,
    /// then attach this Equipment to it.                                      // UNIMPLEMENTED
    ForMirrodin,

    // =========================================================================
    // CR 702.161 — Toxic N
    // =========================================================================
    // PARTIAL: trigger declared; poison-counter delivery partially implemented.
    /// When this creature deals combat damage to a player, that player gets N
    /// poison counters (in addition to the damage). N is stored on CardDef.
    Toxic,

    // =========================================================================
    // CR 702.162 — Backup N
    // =========================================================================
    /// When this creature enters, put N +1/+1 counters on target creature. If
    /// that creature is another creature, it temporarily gains abilities
    /// listed on this card.                                                   // UNIMPLEMENTED
    Backup,

    // =========================================================================
    // CR 702.163 — Bargain
    // =========================================================================
    /// You may sacrifice an artifact, enchantment, or token as you cast this
    /// spell for an additional effect.                                        // UNIMPLEMENTED
    Bargain,

    // =========================================================================
    // CR 702.164 — Craft (cost, material)
    // =========================================================================
    /// Exile this from your graveyard, exile the listed cards from your hand
    /// and/or battlefield: Return this card transformed.                      // UNIMPLEMENTED
    Craft,

    // =========================================================================
    // CR 702.165 — Disguise (cost)
    // =========================================================================
    /// You may cast this card face down as a 2/2 creature with ward {2} for
    /// {3}. Turn it face up any time for its disguise cost.                   // UNIMPLEMENTED
    Disguise,

    // =========================================================================
    // CR 702.166 — Offspring (cost)
    // =========================================================================
    /// You may pay an additional cost when casting this creature spell. If you
    /// do, when this creature enters create a 1/1 token copy of it.           // UNIMPLEMENTED
    Offspring,

    // =========================================================================
    // CR 702.167 — Plot (cost)
    // =========================================================================
    /// Pay cost, Exile this from your hand: Cast it as a sorcery on a later
    /// turn without paying its mana cost.                                     // UNIMPLEMENTED
    Plot,

    // =========================================================================
    // CR 702.168 — Saddle N
    // =========================================================================
    /// Tap any number of other creatures you control with total power N or
    /// more: This Mount becomes saddled until EOT. Saddle only as a sorcery.  // UNIMPLEMENTED
    Saddle,

    // =========================================================================
    // CR 702.169 — Spree
    // =========================================================================
    /// Choose one or more additional costs from the listed modes when casting
    /// this spell.                                                            // UNIMPLEMENTED
    Spree,

    // =========================================================================
    // CR 702.170 — Gift (type)
    // =========================================================================
    /// You may promise an opponent a gift when casting this spell. If you do,
    /// the spell gets an enhanced effect and the opponent receives the gift.  // UNIMPLEMENTED
    Gift,

    // =========================================================================
    // Non-CR "functional keywords" used as flags on CardDef
    // =========================================================================

    /// This creature must attack each combat if able (e.g., Juggernaut).
    MustAttack,
    /// This creature can't block (e.g., Goblin Guide).
    CantBlock,
    /// This creature can't be blocked (e.g., Invisible Stalker, Triton Shorestalker).
    Unblockable,

    // =========================================================================
    // Ward (CR 702.20d — added in MH2 / D&D)
    // =========================================================================
    // PARTIAL: counter-unless-pays-{1} simplified; parameterized costs not enforced.
    /// Whenever this permanent becomes the target of a spell or ability an
    /// opponent controls, counter it unless that opponent pays the ward cost.
    Ward,
}
