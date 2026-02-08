# PR #13 Review: Phase 0 — Shared Interface Contract

**Branch**: `claude/implement-phase-0-NFgI6` -> `mainline`
**Commits**: 3 (`c03b4d9` initial, `2baf8ca` first-pass fixes, `970b70e` second-pass fixes)
**Files changed**: 4 (+1391 / -1)
**Tests**: 46 pass, 0 fail, 0 warnings

---

## Summary

This PR implements Phase 0 of the Consolidated Strategy: the shared interface contract that both Track A (Rules Engine) and Track B (MCCFR Solver) will code against. It adds the Observation API (`PlayerView`), the Canonical Action Mapping (`canonical.rs`), and the Snapshot Contract (documentation on `GameState`).

---

## Review History

| Pass | Commit | Findings | Outcome |
|---|---|---|---|
| 1st | `c03b4d9` | Hand-duplicate disambiguation (3 variants), PlayerView.objects information leak | APPROVE with comments |
| 2nd | `2baf8ca` | Previous fixed. New: incomplete StackSource match, blocker values, timing contract undocumented | APPROVE |
| 3rd | `970b70e` | All previous fixed. No new issues. | **APPROVE — merge-ready** |

---

## Deliverable-by-Deliverable Assessment

### 0.1 — Observation API: `GameState::visible_state(player)` — PASS

- `PlayerView<'a>` struct with all spec fields plus `priority_player` (useful addition).
- `objects` is a **filtered** `HashMap<ObjectId, &'a CardInstance>` — excludes opponent hand and both libraries.
- `visible_state()` exhaustively matches all `StackSource` variants (`Spell`, `ActivatedAbility`, `TriggeredAbility`).
- Combat blockers section inserts both keys (blockers) and values (attackers) for robustness.
- 5 integration tests verify information partitioning.

### 0.2 — Canonical Action Mapping — PASS

- `CanonicalAction` covers all 12 `Action` variants with `Serialize`/`Deserialize`.
- `PlayLand`, `CastSpell`, `Discard` include `hand_index` for duplicate disambiguation.
- `canonicalize()` and `resolve()` document the pre-action timing contract.
- Round-trip invariant holds at strict ObjectId equality, verified by full-game stress test.
- 15 tests (11 unit + 4 integration) covering all action types including 8-duplicate edge case.

### 0.3 — Snapshot Contract — PASS

- Doc comment on `GameState` specifying clone/shared/reconstructed rules.

---

## Acceptance Criteria

| Criterion | Status |
|---|---|
| `PlayerView` struct compiles and `visible_state()` returns it | PASS |
| `canonicalize()` round-trips for all legal actions | PASS |
| Both tracks agree on the interface | N/A (social) |

---

## Third Pass: Fix Verification

### StackSource match — VERIFIED

```rust
let source_id = match entry.source {
    StackSource::Spell(id) => id,
    StackSource::ActivatedAbility { source_id, .. } => source_id,
    StackSource::TriggeredAbility { source_id, .. } => source_id,
};
```

Exhaustive match, correctly extracts the ObjectId from all three variants. For `Spell`, this is the spell card itself (now on the stack zone). For abilities, this is the source permanent. Both are semantically correct — spells on the stack are public, and ability sources should be visible so the solver can reason about what produced the ability.

### Combat blocker values — VERIFIED

```rust
for (&blocker, &attacker) in &self.combat.blockers {
    if let Some(inst) = self.objects.get(&blocker) { visible.insert(blocker, inst); }
    if let Some(inst) = self.objects.get(&attacker) { visible.insert(attacker, inst); }
}
```

Both keys and values inserted. Attackers are typically already present from `combat.attackers`, so the attacker inserts are idempotent (HashMap overwrites are harmless since the same reference is inserted). This removes the fragile dependency.

### Timing contract documentation — VERIFIED

Both `canonicalize()` and `resolve()` now have `# Timing contract` doc sections stating they must be called before `apply_action()`, with clear explanation of what goes wrong otherwise ("incorrect instance indices or panic on missing ObjectIds"). Good.

---

## Remaining Notes (Non-blocking)

1. **Two battlefield disambiguation schemes**: `battlefield_instance_index()` is controller-agnostic; `CanonicalTarget::Object` uses controller-scoped `find_on_battlefield_by_controller()`. Internally consistent but worth a comment for future maintainers. Stylistic only.

2. **`PlayerView.objects` doc comment slightly incomplete**: Mentions "battlefield, stack, both graveyards, both exile zones, the viewing player's hand, and pending trigger sources" but doesn't mention combat participants. Combat participants are battlefield permanents anyway, so the current description is materially correct. Nit.

3. **`StackEntry.targets` not separately added to visible objects**: Stack entry targets (`Target::Object(id)`) are not explicitly collected. In practice, targets are battlefield permanents, graveyard cards, or players — all already in visible zones. If a future card targets something in an unusual zone, this could gap. Very low risk.

---

## Verdict

**APPROVE — merge-ready.** All three second-pass findings are correctly addressed in `970b70e`. The fixes are minimal and targeted (no collateral changes). 46 tests pass. No new issues found.

The PR cleanly delivers all Phase 0 deliverables. Phase 1A and 1B can proceed against this interface.
