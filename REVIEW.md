# PR #18 Review: Phase 2A — Layered Effects Engine, Card Framework & Expanded Pool

## Summary

This PR implements Phase 2A of the MTG GTO simulator across 4 commits:
- **2A.1**: CR 613 layered effects engine (`src/layers/mod.rs`)
- **2A.2**: Composable card framework with new effect types
- **2A.3**: Wire replacement effects into rules engine
- **2A.4**: Expand card pool to 111 cards with comprehensive tests

**+4,039 / -189 lines** across 10 files. All 130 tests pass (45 unit + 70 integration + 14 MCCFR + 1 deck import).

## What Works Well

1. **Layer architecture is clean**: The `ContinuousEffect` / `LayerModification` / `ComputedCharacteristics` model is a sound decomposition of CR 613. Sorting effects by `(layer, timestamp)` and recomputing from scratch on every query is the right approach for correctness-first development.

2. **Static ability → continuous effect pipeline**: The `StaticAbility::to_continuous_effects()` pattern cleanly separates card definitions from runtime effects. `refresh_continuous_effects()` handles the two-phase borrow-checker dance correctly.

3. **Unified keyword/P/T queries via layer engine**: Migrating all `has_keyword()`, `effective_power()`, `effective_toughness()` calls to go through `GameState` → `compute_characteristics()` is a significant improvement over the old scattered `inst.has_keyword(def, ...)` pattern. This makes the combat code substantially cleaner.

4. **Good test coverage**: 20+ new integration tests covering anthems, Humility, Humility+Anthem timestamp ordering, Wrath of God, full games with anthem decks, multicolor cards, etc.

5. **Card pool expansion is impressive**: 111 cards across all five colors, artifacts, and multicolor — with appropriate keywords, effects, and mana costs.

## Bugs

### Bug 1 (High): `is_creature_on_battlefield` doesn't check creature type
**`src/layers/mod.rs:404-411`**

```rust
fn is_creature_on_battlefield(obj_id, objects, battlefield) -> bool {
    battlefield.contains(&obj_id) && objects.get(&obj_id).is_some()
}
```

This function is used by `effect_applies_to()` for `AllCreatures`, `OtherCreatures`, `CreaturesControlledBy`, etc. — but it never checks the object's card type. An anthem effect targeting "all creatures" would also apply to lands, enchantments, and artifacts on the battlefield.

**Fix**: Check `CardType::Creature` from the card definition (or the computed types after layer 4).

### Bug 2 (High): Layer 7d counters applied after layer 7e SwitchPT
**`src/layers/mod.rs:340-342`**

Counters are applied **after** the effects loop, which processes up through layer 7e (SwitchPT). Per CR 613.4, the layer order is 7a → 7b → 7c → **7d** → 7e. Currently, if a creature has +1/+1 counters AND a SwitchPT effect, the counters apply after the switch — producing incorrect results.

**Fix**: Insert counter application inside the loop between 7c and 7e processing, or accumulate 7d effects with a flag and apply them at the right position in the sorted effects list.

### Bug 3 (Medium): `SacrificeCreatures` doesn't let the player choose
**`src/rules/mod.rs:723-733`**

```rust
let creatures = state.creatures_controlled_by(*p);
for &id in creatures.iter().take(*count as usize) { ... }
```

This sacrifices the first N creatures in battlefield order rather than allowing the affected player to choose. Per MTG rules, the player being forced to sacrifice always chooses which creature(s). This should generate a player action (similar to how trigger ordering is handled).

## Issues

### Issue 1 (Performance): `compute_characteristics()` called redundantly
**`src/rules/mod.rs:1308+`, `src/action/mod.rs:547+`**

Every call to `state.has_keyword(id, ...)`, `state.effective_power(id)`, `state.effective_toughness(id)`, and `state.is_creature(id)` calls `compute_characteristics()` from scratch — filtering, sorting, and iterating all active continuous effects each time.

In `resolve_combat_damage()`, a single attacker with blockers triggers 6+ calls. With 5 attackers and 2 blockers each, that's ~90 `compute_characteristics()` calls per combat step. In `check_state_based_actions()`, the SBA loop calls `is_creature()` and `effective_toughness()` per creature, per iteration.

**Suggestion**: Add a per-object characteristics cache that's invalidated when `continuous_effects` changes (i.e., on `refresh_continuous_effects()` or `push()` to the effects list). Or at minimum, compute once per object in hot paths and reuse the `ComputedCharacteristics` struct.

### Issue 2 (Code Quality): Identical branches in `AddKeyword` handler
**`src/layers/mod.rs:293-305`**

```rust
LayerModification::AddKeyword(kw) => {
    if !abilities_removed {
        if !keywords.contains(kw) { keywords.push(*kw); }
    } else {
        // After RemoveAllAbilities, new grants still apply
        if !keywords.contains(kw) { keywords.push(*kw); }
    }
}
```

Both branches are identical. Either collapse them or implement the intended difference.

### Issue 3 (Code Quality): Clippy error — loop that never loops
**`src/action/mod.rs:349`**

`cargo clippy` reports a hard error: "this loop never actually loops" because every arm in the `for ma in &def.mana_abilities` match ends with `break`. This should be rewritten to use `if let Some(first) = def.mana_abilities.first()` or similar.

### Issue 4 (Minor): `can_target_permanent` has unused `_db` parameter
**`src/action/mod.rs:399`**

After migrating to layer-engine-based keyword checks, the `db` parameter to `can_target_permanent` is unused (renamed to `_db`). It should be removed from the signature and all call sites.

### Issue 5 (Minor): `PreventCombatDamage` effect is a no-op
**`src/rules/mod.rs:737-739`**

```rust
Effect::PreventCombatDamage => {
    // Simplified: we don't model this as a replacement effect yet.
}
```

This is a silent no-op. Cards that use this effect (like Fog) will resolve without doing anything. Either implement it or don't include cards that use it until it's implemented.

## Suggestions

1. Consider using `HashSet` instead of `Vec` for `keywords` and `card_types` in `ComputedCharacteristics` — `contains()` checks are frequent and linear scans over small vecs are fine for now, but as the card pool grows this could matter.

2. The `battlefield.contains(&obj_id)` calls in `effect_applies_to()` are O(n) scans. If the battlefield grows large, consider a `HashSet<ObjectId>` mirror or similar.

3. For the MCCFR use case where millions of `GameState::clone()` happen, the current design of recomputing characteristics on every query is correct but may need caching as games get more complex. The comment at the top of `layers/mod.rs` acknowledges this — just flagging it as something to monitor.

## Verdict

**Approve with requested changes** for Bugs 1 and 2 (layer correctness). The architecture is sound, the migration to unified layer-based queries is a clear improvement, and the test coverage is good. The performance concern (Issue 1) is acceptable for now but should be addressed before scaling up MCCFR training.
