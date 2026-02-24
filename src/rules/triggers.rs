use crate::card::{ObjectId, TriggerCondition};
use crate::events::GameEvent;
use crate::game::{GameState, PendingTrigger, PlayerIndex, StackEntry, StackSource};

/// Check all permanents on the battlefield for triggered abilities matching
/// the given condition, and queue any that trigger.
pub(super) fn check_triggers(state: &mut GameState, condition: TriggerCondition, source_hint: Option<ObjectId>) {
    let triggers: Vec<PendingTrigger> = {
        let db = state.card_db();
        let mut found = Vec::new();

        // Determine which objects to check
        let objects_to_check: Vec<(ObjectId, usize)> = match source_hint {
            // If a specific source is given (e.g., ETB on a specific permanent), only check it
            Some(id) => {
                if let Some(inst) = state.objects.get(&id) {
                    vec![(id, inst.controller)]
                } else {
                    vec![]
                }
            }
            // Otherwise check all permanents on the battlefield
            None => state
                .battlefield
                .iter()
                .map(|&id| (id, state.objects[&id].controller))
                .collect(),
        };

        for (obj_id, controller) in objects_to_check {
            let inst = &state.objects[&obj_id];
            let def = match db.get(inst.card_def_id) {
                Some(d) => d,
                None => continue,
            };

            for (i, trigger) in def.triggered_abilities.iter().enumerate() {
                if trigger.trigger == condition {
                    found.push(PendingTrigger {
                        source_id: obj_id,
                        ability_index: i,
                        controller,
                        targets: vec![], // targets chosen when put on stack (simplified: auto-target)
                    });
                }
            }
        }
        found
    };

    state.pending_triggers.extend(triggers);
}

/// Check `ACreatureYouControlDies` triggers — only fires for permanents whose
/// controller matches the dying creature's controller.
pub(super) fn check_your_creature_dies_triggers(
    state: &mut GameState,
    dying_controllers: &[PlayerIndex],
) {
    let triggers: Vec<PendingTrigger> = {
        let db = state.card_db();
        let mut found = Vec::new();

        for &obj_id in &state.battlefield {
            let inst = &state.objects[&obj_id];
            let controller = inst.controller;
            let def = match db.get(inst.card_def_id) {
                Some(d) => d,
                None => continue,
            };

            for (i, trigger) in def.triggered_abilities.iter().enumerate() {
                if trigger.trigger == TriggerCondition::ACreatureYouControlDies
                    && dying_controllers.contains(&controller)
                {
                    found.push(PendingTrigger {
                        source_id: obj_id,
                        ability_index: i,
                        controller,
                        targets: vec![],
                    });
                }
            }
        }
        found
    };

    state.pending_triggers.extend(triggers);
}

/// Flush pending triggers onto the stack in APNAP order
/// (Active Player, Non-Active Player). When a player controls multiple
/// simultaneous triggers, they must choose the ordering — this is surfaced
/// as an `Action::OrderTriggers` decision point for MCCFR to observe.
///
/// If a player has >1 trigger, this function pauses (leaves triggers in
/// `pending_triggers` and sets `priority_player`) so the game loop can
/// present the ordering choice. Returns `true` if all triggers were flushed,
/// `false` if paused waiting for a player's ordering decision.
#[must_use]
pub(super) fn flush_triggers(state: &mut GameState) -> bool {
    if state.pending_triggers.is_empty() {
        return true;
    }

    let active = state.active_player;

    // Count each player's pending triggers
    let ap_count = state
        .pending_triggers
        .iter()
        .filter(|t| t.controller == active)
        .count();
    let nap_count = state
        .pending_triggers
        .iter()
        .filter(|t| t.controller != active)
        .count();

    // APNAP: handle active player's triggers first
    if ap_count > 1 {
        // AP has multiple triggers — pause for ordering decision
        state.priority_player = active;
        return false;
    }

    // AP has 0-1 triggers: auto-push them to the stack
    let ap_triggers: Vec<PendingTrigger> = state
        .pending_triggers
        .iter()
        .filter(|t| t.controller == active)
        .cloned()
        .collect();
    for trigger in ap_triggers {
        push_trigger_to_stack(state, &trigger);
    }
    state.pending_triggers.retain(|t| t.controller != active);

    // Now handle non-active player's triggers
    if nap_count > 1 {
        // NAP has multiple triggers — pause for ordering decision
        let nap = state.opponent(active);
        state.priority_player = nap;
        return false;
    }

    // NAP has 0-1 triggers: auto-push them
    let nap_triggers = std::mem::take(&mut state.pending_triggers);
    for trigger in nap_triggers {
        push_trigger_to_stack(state, &trigger);
    }

    true
}

/// Push a single trigger onto the stack as a TriggeredAbility entry.
pub(super) fn push_trigger_to_stack(state: &mut GameState, trigger: &PendingTrigger) {
    let stack_id = state.new_stack_id();
    state.stack.push(StackEntry {
        id: stack_id,
        source: StackSource::TriggeredAbility {
            source_id: trigger.source_id,
            ability_index: trigger.ability_index,
        },
        controller: trigger.controller,
        targets: trigger.targets.clone(),
    });
    state.emit_event(GameEvent::AbilityTriggered {
        source: trigger.source_id,
        ability_index: trigger.ability_index,
    });
}

/// Check triggers for a specific game event and flush them to the stack.
/// Returns `true` if all triggers were flushed, `false` if paused waiting
/// for a player's ordering decision (i.e. pending_triggers is non-empty).
#[must_use]
pub fn fire_triggers(state: &mut GameState, condition: TriggerCondition, source_hint: Option<ObjectId>) -> bool {
    check_triggers(state, condition, source_hint);
    flush_triggers(state)
}

/// Count how many permanents a player controls that have the
/// `ManaFromNonlandBonus` static ability (e.g., Kinnan, Bonder Prodigy).
/// Returns the total bonus amount (typically 1 per source).
pub(super) fn mana_from_nonland_bonus_count(state: &GameState, player: PlayerIndex) -> u32 {
    let db = state.card_db();
    let mut count = 0u32;
    for &obj_id in &state.battlefield {
        let inst = match state.objects.get(&obj_id) {
            Some(i) => i,
            None => continue,
        };
        if inst.controller != player {
            continue;
        }
        let def = match db.get(inst.card_def_id) {
            Some(d) => d,
            None => continue,
        };
        for sa in &def.static_abilities {
            if matches!(sa, crate::layers::StaticAbility::ManaFromNonlandBonus) {
                count += 1;
            }
        }
    }
    count
}

/// Fire spell-cast triggers for a spell that was just cast.
/// `caster` is the player who cast the spell. `is_creature` indicates whether
/// the spell is a creature spell (relevant for OpponentCastsNoncreatureSpell).
pub(super) fn fire_spell_cast_triggers(state: &mut GameState, caster: PlayerIndex, is_creature: bool) {
    let triggers: Vec<PendingTrigger> = {
        let db = state.card_db();
        let mut found = Vec::new();

        for &obj_id in &state.battlefield {
            let inst = &state.objects[&obj_id];
            let controller = inst.controller;
            let def = match db.get(inst.card_def_id) {
                Some(d) => d,
                None => continue,
            };

            for (i, trigger) in def.triggered_abilities.iter().enumerate() {
                let matches = match trigger.trigger {
                    TriggerCondition::YouCastSpell => controller == caster,
                    TriggerCondition::YouCastCreatureSpell => {
                        controller == caster && is_creature
                    }
                    TriggerCondition::OpponentCastsSpell => controller != caster,
                    TriggerCondition::OpponentCastsNoncreatureSpell => {
                        controller != caster && !is_creature
                    }
                    _ => false,
                };
                if matches {
                    found.push(PendingTrigger {
                        source_id: obj_id,
                        ability_index: i,
                        controller,
                        targets: vec![],
                    });
                }
            }
        }
        found
    };

    state.pending_triggers.extend(triggers);
    let _ = flush_triggers(state);
}

/// Fire card-draw triggers when a player draws a card.
/// `drawing_player` is the player who drew. This fires `OpponentDrawsCard`
/// on permanents controlled by each opponent of the drawing player.
pub(super) fn fire_card_draw_triggers(state: &mut GameState, drawing_player: PlayerIndex) {
    let triggers: Vec<PendingTrigger> = {
        let db = state.card_db();
        let mut found = Vec::new();

        for &obj_id in &state.battlefield {
            let inst = &state.objects[&obj_id];
            let controller = inst.controller;
            let def = match db.get(inst.card_def_id) {
                Some(d) => d,
                None => continue,
            };

            for (i, trigger) in def.triggered_abilities.iter().enumerate() {
                if trigger.trigger == TriggerCondition::OpponentDrawsCard
                    && controller != drawing_player
                {
                    found.push(PendingTrigger {
                        source_id: obj_id,
                        ability_index: i,
                        controller,
                        targets: vec![],
                    });
                }
            }
        }
        found
    };

    state.pending_triggers.extend(triggers);
    let _ = flush_triggers(state);
}
