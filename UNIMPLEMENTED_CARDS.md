# Unimplemented Cards

Cards in the sample database that still use `Effect::Unimplemented` because
they require engine features that do not yet exist.

## Semblance Anvil

- **Deck:** Ashcoat / Skitter
- **Card type:** Artifact ({3})
- **Ability:** Imprint — When Semblance Anvil enters the battlefield, you may exile a nonland card from your hand. Spells you cast that share a card type with the exiled card cost {2} less to cast.
- **Blocker:** Requires the imprint mechanic (exile a card linked to a permanent, then reference its properties for a continuous cost-reduction effect). Needs linked exile zones and dynamic cost reduction based on exiled card's types.

## Strionic Resonator

- **Deck:** Ashcoat / Skitter
- **Card type:** Artifact ({2})
- **Ability:** {2}, {T}: Copy target triggered ability you control. You may choose new targets for the copy.
- **Blocker:** Requires the ability to copy triggered abilities on the stack. The engine would need to duplicate a stack entry (trigger) and allow re-targeting, which is not currently supported.

## Swarmyard

- **Deck:** Ashcoat / Skitter
- **Card type:** Land
- **Ability:** {T}: Regenerate target Insect, Rat, Spider, or Squirrel.
- **Blocker:** Requires the regenerate mechanic (CR 701.15). Regeneration replaces the next destruction event on a permanent with: tap it, remove all damage, remove it from combat. This replacement-effect-based protection is not modeled in the engine.

## Ink-Eyes, Servant of Oni (regenerate ability only)

- **Deck:** Ashcoat / Skitter
- **Card type:** Creature — Rat Ninja ({4}{B}{B})
- **Ability:** {1}{B}: Regenerate Ink-Eyes, Servant of Oni.
- **Blocker:** Same as Swarmyard — requires the regenerate mechanic (CR 701.15). The combat damage trigger is fully implemented.

---

*Note:* The Scryfall auto-importer (`src/scryfall.rs`) also falls back to
`Effect::Unimplemented` for cards it cannot parse automatically. The five
cards listed above are the only hand-authored definitions that remain
unimplemented.
