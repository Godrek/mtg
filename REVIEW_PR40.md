# PR #40 Review: Add card catalog abstraction and expand modeled card effects

## Summary

This PR introduces a central card catalog (`src/card/catalog.rs`) that provides a
canonical mapping of card IDs to names/keys, extracts card ID constants from
`sample.rs` into a shared location, adds a `card_implementation_status()` query
function, implements Shivan Dragon's firebreathing ability, fixes Brainstorm's draw
count, and applies `rustfmt` across the codebase.

**Build: PASS** | **Tests: 268/268 PASS** | **Clippy: 69 warnings (all pre-existing)**

## Changes Reviewed

| File | Lines | Nature |
|------|-------|--------|
| `src/card/catalog.rs` | +1287 | New: Card catalog with IDs, `ALL_CARDS` array, `card_implementation_status()` |
| `src/card/mod.rs` | +93/-68 | `pub mod catalog` + rustfmt reformatting |
| `src/card/sample.rs` | +131/-391 | Re-export IDs from catalog, Shivan Dragon firebreathing, Brainstorm fix, rustfmt |
| `tests/commander_test.rs` | +39/-25 | Rustfmt, remove unused variable |
| `tests/integration_test.rs` | +571/-388 | 3 new tests, rustfmt, import reordering |

## Positive Aspects

1. **Centralizes card IDs**: Moving the `ids` module from `sample.rs` to `catalog.rs`
   is a good step toward separating the card registry from sample deck construction.
   The `pub use` re-export in `sample::ids` preserves backward compatibility.

2. **`card_implementation_status()`**: Useful for tracking which cards have modeled
   effects vs. vanilla stat-blocks. Good for project health dashboards.

3. **Shivan Dragon fix**: Adding the `{R}: +1/+0` firebreathing ability is a correct
   rules improvement over the previous `activated_abilities: vec![]` stub.

4. **Brainstorm fix**: Changing from `DrawCards { count: 1 }` to `DrawCards { count: 3 }`
   is more accurate. The comment correctly notes the "put 2 back" part is still
   simplified away.

5. **New tests**: `test_catalog_ids_match_sample_ids`, `test_catalog_effect_status_marks_effect_cards`,
   and `test_shivan_dragon_has_firebreathing_ability` all properly validate the new code.

## Issues

### Medium: Catalog names diverge from CardDef names

Several `CardCatalogEntry::name` values don't match the actual `CardDef::name` used
in `build_sample_db()`. For example:

- Catalog: `"Thalia Guardian"` vs CardDef: `"Thalia, Guardian of Thraben"`
- Catalog: `"Brimaz King"` vs CardDef: `"Brimaz, King of Oreskos"`
- Catalog: `"the One Ring"` (lowercase 't') vs likely intended `"The One Ring"`
- Catalog: `"an Offer You Cant Refuse"` (lowercase 'a', missing apostrophe)
- Catalog: `"Kinnan Bonder Prodigy"` vs CardDef: `"Kinnan, Bonder Prodigy"`
- Catalog: `"Green Suns Zenith"` vs correct: `"Green Sun's Zenith"`

These mismatches could cause bugs if catalog names are ever used for display or
lookup. Consider generating catalog entries from the DB or adding a test that
validates `ALL_CARDS[i].name == db.get(ALL_CARDS[i].id).unwrap().name`.

### Medium: `effects_implemented` misclassifies mana-producing cards

`card_implementation_status()` checks for `spell_effect`, `activated_abilities`,
`triggered_abilities`, and `static_abilities` — but not `mana_abilities`. This means
fully-functional mana rocks and dorks (Sol Ring, Llanowar Elves, Basalt Monolith, etc.)
show as "not implemented" even though they work correctly. Consider including
`!def.mana_abilities.is_empty()` in the check, or renaming the field to clarify it
means "non-mana effects implemented."

### Low: Formatting changes dominate the diff

~80% of the line changes are `rustfmt` reformatting, making it harder to identify
the functional changes (catalog extraction, Shivan Dragon ability, Brainstorm fix).
Consider splitting the PR into:
1. `rustfmt` pass (pure formatting)
2. Card catalog + functional card fixes

This would make the functional changes easier to review and bisect.

### Low: `CardCatalogEntry` and `ALL_CARDS` are compile-time only

Using `&'static str` for keys and names means the catalog can't incorporate
runtime-imported cards (e.g., from Scryfall). This is fine for now but worth noting
as a future constraint if the system moves toward dynamic card loading.

### Nit: Unused `CardCatalogEntry::key` field

The `key` field (e.g., `"LIGHTNING_BOLT"`) duplicates the constant name from the
`ids` module but isn't used anywhere in the codebase yet. If it's intended for
future serialization or lookup, that's fine — otherwise it's dead code.

## Verdict

The PR is **functionally correct** — it builds cleanly and all 268 tests pass. The
catalog abstraction is a reasonable step toward better card data organization. The
name mismatches and `effects_implemented` classification should be addressed before
merge to prevent latent bugs. The formatting changes are harmless but would benefit
from being a separate commit.

**Recommendation: Approve with requested changes** (fix catalog name mismatches,
include `mana_abilities` in implementation status check).
