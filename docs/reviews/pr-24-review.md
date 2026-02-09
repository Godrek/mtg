# PR #24 Review: Fix mana verification, commander setup, ManaCost parsing, and magic number

**Branch:** `claude/fix-mana-commander-setup-l22Rb`
**Commit:** `591761b` (rebased)
**Author:** Claude (automated)

## Round 1 — Request Changes (commit `00acb51`)

The initial PR bundled five changes, but parts conflicted with mainline's commander model. Six items were flagged:

1. Extend mana guard fix to `CastCommander` and `ActivateAbility` (not just `CastSpell`)
2. Drop duplicate commander fields (`command_zone`, `commander_damage: HashMap`) — mainline already has a more complete model
3. Drop thin `setup_commander_game()` wrapper — mainline's version is strictly better
4. Fix `colors()` to actually include hybrid/phyrexian colors (doc said it did, code didn't)
5. Rebase onto mainline
6. Rewrite commander snapshot test to use mainline's commander API

## Round 2 — Approve (commit `591761b`)

All six items have been addressed. Changes verified:

### 1. Mana guard extended to all three handlers

`src/rules/mod.rs` — all three `pay()` call sites now check the return value and abort on failure:
- `CastSpell` (line 76): `if !state.players[player].mana_pool.pay(cost) { return; }`
- `ActivateAbility` (line 166): `if !state.players[player].mana_pool.pay(&ability.cost) { return; }`
- `CastCommander` (line 302): `if !state.players[player].mana_pool.pay(&taxed_cost) { return; }`

### 2. Commander field changes dropped

`src/game/mod.rs` is no longer modified. No conflicting `commander_damage: HashMap<ObjectId, u32>` or duplicate `command_zone` additions.

### 3. `setup_commander_game()` dropped

No new `setup_commander_game()` added. The test uses mainline's existing implementation.

### 4. `colors()` fixed to include hybrid/phyrexian

`src/mana/mod.rs` — `colors()` now uses a `HashSet<Color>` and collects from:
- Regular colored mana fields (white, blue, black, red, green)
- Both colors from each hybrid symbol
- Each phyrexian symbol's color

Doc comment and implementation are now consistent.

### 5. Rebased onto mainline

PR is now based on `origin/mainline` (`47dd5ee`). Single clean commit on top.

### 6. Commander snapshot test rewritten

`tests/integration_test.rs` — `test_commander_snapshot_restore` now uses:
- `GameState::new_commander(2)` (mainline API)
- `sample::brimaz_commander_deck()` / `sample::thrun_commander_deck()` (mainline decks)
- Verifies round-trip of all mainline commander fields: `command_zone`, `commander_card_id`, `commander_object_id`, `commander_tax`, `commander_damage_received`

### Remaining changes (all approved)

| Change | File | Status |
|--------|------|--------|
| ManaCost X/hybrid/phyrexian parsing | `src/mana/mod.rs` | New feature, well-tested |
| `DEFAULT_MAX_ACTIONS` constant | `src/solver/mccfr.rs` | Clean refactor |
| Plains land fix in SBA test | `tests/integration_test.rs` | Bug fix (test was passing only because of mana guard bug) |

### Test results

All 221 tests pass (64 unit + 103 integration + 25 MCCFR + 24 lib + 4 commander + 1 goldfish).

## Verdict: Approve
