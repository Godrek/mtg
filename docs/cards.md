# Card System

How cards are defined, loaded, and extended.

## Card Definitions

Cards are defined as `CardDef` structs with composable data fields -- no bespoke code per card.

Key fields:
- `name`, `mana_cost`, `card_types`, `subtypes`
- `power`, `toughness`, `starting_loyalty`
- `keywords: Vec<KeywordAbility>`
- `effects: Vec<Effect>` (spell effects on resolution)
- `triggered_abilities: Vec<TriggeredAbility>`
- `activated_abilities: Vec<ActivatedAbility>`
- `static_abilities: Vec<StaticAbility>`
- `loyalty_abilities: Vec<LoyaltyAbility>`
- `equip_cost`, `enters_tapped`, `is_legendary`
- `color_identity()` derived from mana cost + abilities

## Hand-Authored Cards (275+)

Defined in `src/card/sample.rs`. These are fully implemented and tested.

### Adding a New Card

1. Add a constant ID in the `ids` module:
   ```rust
   pub const MY_CARD: CardId = 999;
   ```

2. Insert the `CardDef` in `build_sample_db()`:
   ```rust
   db.insert(ids::MY_CARD, CardDef {
       name: "My Card".to_string(),
       mana_cost: ManaCost::parse("{2}{G}"),
       card_types: vec![CardType::Creature],
       subtypes: vec!["Beast".to_string()],
       power: Some(3),
       toughness: Some(3),
       enters_tapped: false,
       ..CardDef::default()
   });
   ```

3. Use existing cards as templates for complex abilities.

## Scryfall Auto-Parse

Cards not in the sample database are fetched from the Scryfall API (behind the `scryfall` feature flag) and auto-parsed:

- Oracle text is parsed for keywords, triggered abilities, activated abilities, static abilities, and effects
- Parsing handles ~40+ patterns including ETB triggers, damage effects, token creation, cost reduction, equip costs
- Auto-parsed cards work in goldfish mode; complex interactions may be stubbed as `Effect::Unimplemented`
- Results are cached to disk to avoid repeated API calls

## Two-Tier Resolution

When loading a deck:
1. **Tier 1**: Check the hand-authored sample database (275+ cards, fully tested)
2. **Tier 2**: Fall back to Scryfall API fetch + auto-parse oracle text

This means any Commander deck can be loaded -- known cards get full fidelity, unknown cards get best-effort parsing.

## Card Coverage Tracking

```bash
cargo run --release --bin goldfish -- --deck my_deck.txt --coverage
```

Reports per-card coverage level:
- `FullyImplemented` -- in sample database, no stub effects
- `AutoParsed` -- loaded via Scryfall, auto-parsed
- `Stubbed` -- has `Effect::Unimplemented` effects
- `Unknown` -- couldn't be resolved

## Commander Prebuilt Decks

Four 100-card Commander decks in `src/card/sample.rs`:

| Deck | Commander | Strategy |
|------|-----------|----------|
| Kinnan | Kinnan, Bonder Prodigy | Simic mana combo (Basalt Monolith infinite) |
| Brimaz | Brimaz, King of Oreskos | Mono-white tokens/aggro |
| Ashcoat | Ashcoat of the Shadow Swarm | Mono-black rat tribal + Thornbite combos |
| Thrun | Thrun, Breaker of Silence | Mono-green voltron |

Deck files can also be loaded from `decks/` directory (text format, one card per line).

## Effect System

Effects are the building blocks for card behaviors. 42+ variants including:

- **Damage/Life**: `DealDamage`, `GainLife`, `LoseLife`, `EachOpponentLosesLife`, `GainDynamicLife`
- **Cards**: `DrawCards`, `DiscardCards`, `MillCards`, `Scry`, `DrawThenDiscard`
- **Removal**: `DestroyTarget`, `DestroyAllCreatures`, `ExileTarget`, `SacrificeCreatures`
- **Buff**: `Buff`, `Debuff`, `SetPowerToughness`, `GainKeywordUntilEOT`
- **Tokens**: `CreateToken`, `CreateTokens`, `CreatePredefinedToken` (Treasure, Food, Clue, etc.)
- **Zones**: `BounceTo`, `ReturnFromGraveyardToBattlefield`, `ReturnFromGraveyardToHand`, `ExileFromGraveyard`, `ShuffleIntoLibrary`
- **Control**: `GainControlUntilEOT`, `TapTarget`, `Fight`
- **Mana**: `AddMana`
- **Counter**: `CounterSpell`
- **Modal/Conditional**: `Modal`, `Conditional`, `ForEach`

## Predefined Token Types

11 predefined token templates for common token patterns:

Treasure, Food, Clue, Blood, Map, Powerstone, Shard, Soldier11, Spirit11, Zombie22, Beast33
