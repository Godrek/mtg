# Rules Engine

Comprehensive coverage of implemented MTG rules for Commander goldfish play.

## Turn Structure

Full 13-phase turn structure:
1. Untap
2. Upkeep
3. Draw
4. Pre-Combat Main Phase
5. Beginning of Combat
6. Declare Attackers
7. Declare Blockers
8. First Strike Damage
9. Combat Damage
10. End of Combat
11. Post-Combat Main Phase
12. End Step
13. Cleanup

Priority system with consecutive pass tracking. Extra turns queue and skip phases set.

## Stack & Resolution

- Spells, activated abilities, and triggered abilities use the stack
- LIFO resolution order
- Priority passes between each resolution
- Counterspell support
- X spell handling (`ManaCost::x_count`)

## Combat

- Declare attackers/blockers with subset enumeration (capped at 10 attackers, 6 blockers per attacker)
- First strike and double strike damage steps
- Trample (excess damage to defending player)
- Deathtouch (1 damage lethal to blockers)
- Lifelink (damage heals controller)
- Vigilance (doesn't tap to attack)
- Flying / Reach blocking restrictions
- Fear, Intimidate, Menace, Shadow, Horsemanship, Skulk evasion
- Flanking (-1/-1 to blockers)
- Unblockable, Landwalk
- MustAttack and CantBlock enforcement

## State-Based Actions (CR 704.5)

Run in a loop until stable, then flush triggers:
- Lethal damage (toughness <= 0 or damage >= toughness)
- Zero or less life
- Commander damage >= 21
- Legendary rule (keep newest, destroy rest)
- Planeswalker uniqueness
- +1/+1 and -1/-1 counter cancellation
- Aura / Equipment attachment cleanup
- Token cleanup (tokens cease to exist when leaving battlefield)
- Empty library (draw from empty = lose)

## Trigger System

19 trigger conditions:
- ETB (enters the battlefield) -- self and watcher
- Dies -- self and watcher (`ACreatureDies`)
- Attacks, Blocks
- Deals damage, Deals combat damage
- Upkeep, End step
- Spell cast, Noncreature spell cast
- Card drawn
- Creature enters (`ACreatureEnters`)
- And more

APNAP ordering (active player's triggers go on stack first). `OrderTriggers` action for player-controlled ordering.

## Keywords (34+)

**Combat:** Flying, Reach, First Strike, Double Strike, Trample, Deathtouch, Lifelink, Vigilance, Haste, Defender, Menace, Fear, Intimidate, Shadow, Horsemanship, Skulk, Flanking, Unblockable

**Evasion:** Landwalk variants (Plains, Island, Swamp, Mountain, Forest)

**Damage:** Wither, Infect, Toxic(N)

**Triggered:** Prowess, Undying, Persist, Annihilator(N), Exalted, Extort, Cascade, Storm

**Cost modifiers:** Affinity for Artifacts, Convoke, Delve, Flash

**Alternative costs:** Flashback, Escape

**Other:** Changeling, Devoid, Indestructible, Hexproof, Shroud, Protection (simplified), Ward (simplified), Partner, MustAttack, CantBlock

## Effects (42+)

Damage, life gain/loss, draw, discard, destroy, exile, bounce, buff/debuff, counter spells, create tokens (11 predefined token types), mill, scry, fight, tap, gain control, gain keyword, sacrifice, and more. Modal and conditional effects supported.

## Continuous Effects (CR 613)

Full layer system:
- Layer 1: Copy effects
- Layer 2: Control-changing effects
- Layer 3: Text-changing effects
- Layer 4: Type-changing effects
- Layer 5: Color-changing effects
- Layer 6: Ability-adding/removing effects
- Layer 7a-e: P/T setting and modifying effects (including counters)

Timestamp ordering for conflicting effects within the same layer.

## Replacement Effects (CR 614)

- ETB replacements (enters tapped, enters with counters)
- Death replacements (Undying, Persist, commander to command zone)
- Damage prevention / redirection
- Self-replacement priority (CR 614.16a)
- `ChooseReplacementOrder` action for player-controlled ordering

## Commander Rules

- Command zone with commander casting
- Commander tax (+2 generic per previous cast)
- Commander damage tracking (21 = lethal, per commander)
- Color identity enforcement (mana cost + mana abilities)
- Singleton validation (no duplicates except basic lands)
- Partner commanders (separate tax tracking)
- Commander death/exile replacement (auto-redirect to command zone)
- 40 starting life

## Mana System

- WUBRG + Colorless + Any mana types
- ManaCost parsing from string format (`"{2}{W}{U}"`)
- Smart auto-tap: most-constrained-first algorithm
  - Constraint scoring: single-color (1) > dual (2) > colorless (50) > any (100)
  - Pass 1: Pay colored costs from most constrained sources
  - Pass 2: Pay generic costs from colorless-only sources first
- Mana rocks and dorks tapped alongside lands
- Cost reduction (Affinity, Convoke, Delve, static reducers)
- Cost increase (Thalia-style tax effects)

## Not Yet Implemented

- Copy effects (Clone, Fork, Twincast)
- Face-down cards (Morph, Manifest, Foretell)
- Full multi-player (4-player) -- structural support exists but goldfish is the focus
- Full Protection implementation (simplified)
- Kicker, Overload, Evoke interactive decisions
