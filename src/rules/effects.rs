use crate::card::{Effect, KeywordAbility, ObjectId, TriggerCondition, ZoneType};
use crate::events::GameEvent;
use crate::game::{GameState, PlayerIndex, StackSource, Target};

/// Resolve an effect.
pub(super) fn resolve_effect(
    state: &mut GameState,
    effect: &Effect,
    controller: PlayerIndex,
    targets: &[Target],
) {
    match effect {
        Effect::DealDamage { amount, target: target_spec } => {
            // For untargeted effects, auto-generate targets from the spec.
            let effective_targets: Vec<Target> = if targets.is_empty() {
                match target_spec {
                    crate::card::TargetSpec::NoTarget => {
                        // "Each player" — deal damage to all players
                        (0..state.players.len())
                            .map(|i| Target::Player(i))
                            .collect()
                    }
                    crate::card::TargetSpec::EachCreature => {
                        // "Each creature" — deal damage to all creatures on the battlefield
                        let db = state.card_db();
                        state.battlefield.iter().copied()
                            .filter(|&id| {
                                state.objects.get(&id)
                                    .and_then(|inst| db.get(inst.card_def_id))
                                    .map_or(false, |def| def.is_creature())
                            })
                            .map(Target::Object)
                            .collect()
                    }
                    _ => vec![],
                }
            } else {
                targets.to_vec()
            };

            for target in &effective_targets {
                // Apply replacement effects to damage (CR 614)
                let actual_damage = state.deal_damage_with_replacement(*amount, target);
                if actual_damage == 0 {
                    continue;
                }

                match target {
                    Target::Player(p) => {
                        let old_life = state.players[*p].life;
                        state.players[*p].life -= actual_damage as i32;
                        state.emit_event(GameEvent::LifeChanged {
                            player: *p,
                            old: old_life,
                            new: state.players[*p].life,
                        });
                        state.emit_event(GameEvent::DamageDealt {
                            source: 0,
                            target: target.clone(),
                            amount: actual_damage,
                            is_combat: false,
                        });
                    }
                    Target::Object(id) => {
                        if let Some(inst) = state.objects.get_mut(id) {
                            inst.damage_marked += actual_damage;
                        }
                        state.emit_event(GameEvent::DamageDealt {
                            source: 0,
                            target: target.clone(),
                            amount: actual_damage,
                            is_combat: false,
                        });
                    }
                }
            }
        }

        Effect::GainLife { amount } => {
            let old_life = state.players[controller].life;
            state.players[controller].life += *amount as i32;
            state.emit_event(GameEvent::LifeChanged {
                player: controller,
                old: old_life,
                new: state.players[controller].life,
            });
        }

        Effect::LoseLife { amount, .. } => {
            for target in targets {
                if let Target::Player(p) = target {
                    let old_life = state.players[*p].life;
                    state.players[*p].life -= *amount as i32;
                    state.emit_event(GameEvent::LifeChanged {
                        player: *p,
                        old: old_life,
                        new: state.players[*p].life,
                    });
                }
            }
        }

        Effect::DrawCards { count } => {
            super::draw_cards(state, controller, *count as usize);
        }

        Effect::DestroyTarget { .. } => {
            // Collect which objects are destroyable (read phase)
            let destroyable: Vec<ObjectId> = targets
                .iter()
                .filter_map(|target| {
                    if let Target::Object(id) = target {
                        let indestructible =
                            state.has_keyword(*id, KeywordAbility::Indestructible);
                        if !indestructible { Some(*id) } else { None }
                    } else {
                        None
                    }
                })
                .collect();

            for &id in &destroyable {
                state.move_object(id, ZoneType::Battlefield, ZoneType::Graveyard);
            }
            if !destroyable.is_empty() {
                state.refresh_continuous_effects();
            }
            // Fire dies triggers for destroyed creatures.
            for &id in &destroyable {
                super::triggers::check_triggers(state, TriggerCondition::Dies, Some(id));
            }
            if !destroyable.is_empty() {
                let _ = super::triggers::flush_triggers(state);
            }
        }

        Effect::BounceTo { zone, .. } => {
            for target in targets {
                if let Target::Object(id) = target {
                    state.move_object(*id, ZoneType::Battlefield, *zone);
                }
            }
        }

        Effect::Buff {
            power,
            toughness,
            until_eot,
        } => {
            use crate::layers::{AffectedObjects, ContinuousEffect, Duration, LayerModification};
            for target in targets {
                if let Target::Object(id) = target {
                    if *until_eot {
                        // Create a continuous effect that lasts until end of turn
                        let ts = state.new_timestamp();
                        state.continuous_effects.push(ContinuousEffect {
                            source_id: *id,
                            controller,
                            timestamp: ts,
                            duration: Duration::UntilEndOfTurn,
                            affected: AffectedObjects::Specific(*id),
                            modification: LayerModification::ModifyPT(*power, *toughness),
                        });
                    } else {
                        // Permanent buff via +1/+1 counters
                        if let Some(inst) = state.objects.get_mut(id) {
                            let counters = (*power).min(*toughness);
                            inst.plus_counters += counters;
                            // Any asymmetric remainder as a permanent continuous effect
                            if *power != *toughness {
                                let ts = state.new_timestamp();
                                state.continuous_effects.push(ContinuousEffect {
                                    source_id: *id,
                                    controller,
                                    timestamp: ts,
                                    duration: Duration::Permanent,
                                    affected: AffectedObjects::Specific(*id),
                                    modification: LayerModification::ModifyPT(
                                        power - counters,
                                        toughness - counters,
                                    ),
                                });
                            }
                        }
                    }
                }
            }
            state.invalidate_characteristics_cache();
        }

        Effect::DiscardCards { count, .. } => {
            for target in targets {
                if let Target::Player(p) = target {
                    super::discard_random(state, *p, *count as usize);
                }
            }
        }

        Effect::CreateToken(token_def) => {
            super::tokens::create_token(state, token_def, controller);
        }

        Effect::CreateTokens { token, count } => {
            let db_ref = state.card_db.clone();
            let db = db_ref.as_ref().expect("card_db required");
            let n = count.evaluate(
                controller,
                &state.objects,
                &state.battlefield,
                &|id| db.get(id),
                None,
            );
            for _ in 0..n.max(0) {
                super::tokens::create_token(state, token, controller);
            }
        }

        Effect::Counter { .. } => {
            // Find the targeted spell on the stack and counter it
            let target_obj_id = targets.iter().find_map(|t| {
                if let Target::Object(id) = t { Some(*id) } else { None }
            });

            if let Some(target_id) = target_obj_id {
                // Find and remove the targeted spell from the stack
                if let Some(idx) = state.stack.iter().position(|entry| {
                    matches!(&entry.source, StackSource::Spell(id) if *id == target_id)
                }) {
                    let countered = state.stack.remove(idx);
                    if let StackSource::Spell(obj_id) = countered.source {
                        state.move_object(obj_id, ZoneType::Stack, ZoneType::Graveyard);
                    }
                }
            } else {
                // Fallback: counter top spell on stack if no target specified
                if let Some(countered) = state.stack.pop() {
                    if let StackSource::Spell(obj_id) = countered.source {
                        state.move_object(obj_id, ZoneType::Stack, ZoneType::Graveyard);
                    }
                }
            }
        }

        Effect::ExileTarget { .. } => {
            for target in targets {
                if let Target::Object(id) = target {
                    if state.battlefield.contains(id) {
                        state.move_object(*id, ZoneType::Battlefield, ZoneType::Exile);
                    }
                }
            }
            state.refresh_continuous_effects();
            state.refresh_replacement_effects();
        }

        Effect::DestroyAll => {
            // Destroy all creatures on the battlefield (e.g., Wrath of God)
            let creatures: Vec<ObjectId> = state
                .battlefield
                .iter()
                .copied()
                .filter(|&id| state.is_creature(id))
                .filter(|&id| !state.has_keyword(id, KeywordAbility::Indestructible))
                .collect();

            for &id in &creatures {
                let dest_zone = state.death_replacement_zone(id);
                if dest_zone != ZoneType::Battlefield {
                    state.move_object(id, ZoneType::Battlefield, dest_zone);
                }
            }
            if !creatures.is_empty() {
                state.refresh_continuous_effects();
                state.refresh_replacement_effects();
            }
            // Fire dies triggers
            for &id in &creatures {
                super::triggers::check_triggers(state, TriggerCondition::Dies, Some(id));
            }
            if !creatures.is_empty() {
                let _ = super::triggers::flush_triggers(state);
            }
        }

        Effect::Debuff {
            power,
            toughness,
            until_eot,
        } => {
            use crate::layers::{AffectedObjects, ContinuousEffect, Duration, LayerModification};
            for target in targets {
                if let Target::Object(id) = target {
                    let duration = if *until_eot {
                        Duration::UntilEndOfTurn
                    } else {
                        Duration::Permanent
                    };
                    let ts = state.new_timestamp();
                    state.continuous_effects.push(ContinuousEffect {
                        source_id: *id,
                        controller,
                        timestamp: ts,
                        duration,
                        affected: AffectedObjects::Specific(*id),
                        modification: LayerModification::ModifyPT(-power, -toughness),
                    });
                }
            }
            state.invalidate_characteristics_cache();
        }

        Effect::PutCounters { count, .. } => {
            for target in targets {
                if let Target::Object(id) = target {
                    if let Some(inst) = state.objects.get_mut(id) {
                        if *count > 0 {
                            inst.plus_counters += count;
                        } else {
                            inst.minus_counters += count.abs();
                        }
                    }
                }
            }
            state.invalidate_characteristics_cache();
        }

        Effect::MillCards { count, .. } => {
            for target in targets {
                if let Target::Player(p) = target {
                    for _ in 0..*count {
                        if let Some(card_id) = state.players[*p].library.pop() {
                            state.move_object(card_id, ZoneType::Library, ZoneType::Graveyard);
                        }
                    }
                }
            }
        }

        Effect::SacrificeCreatures { count, .. } => {
            for target in targets {
                if let Target::Player(p) = target {
                    let mut creatures = state.creatures_controlled_by(*p);
                    // Sort by effective power ascending so the weakest are
                    // sacrificed first — a reasonable heuristic standing in
                    // for actual player choice until we surface a UI action.
                    creatures.sort_by_key(|&id| state.effective_power(id));
                    for &id in creatures.iter().take(*count as usize) {
                        state.move_object(id, ZoneType::Battlefield, ZoneType::Graveyard);
                    }
                }
            }
            state.refresh_continuous_effects();
        }

        Effect::PreventCombatDamage => {
            // Not yet implemented — no cards in the current pool use this effect.
        }

        Effect::AddMana { color, amount } => {
            for _ in 0..*amount {
                match color {
                    Some(c) => state.players[controller].mana_pool.add_color(*c, 1),
                    None => state.players[controller].mana_pool.colorless += 1,
                }
            }
        }

        Effect::AddDynamicMana { color, count } => {
            let db_ref = state.card_db.clone();
            let db = db_ref.as_ref().expect("card_db required");
            let ctx = super::tokens::build_dynamic_context(state, controller);
            let n = count.evaluate(
                controller,
                &state.objects,
                &state.battlefield,
                &|id| db.get(id),
                Some(&ctx),
            );
            for _ in 0..n.max(0) {
                state.players[controller].mana_pool.add_color(*color, 1);
            }
        }

        Effect::LoseDynamicLife { amount, .. } => {
            let db_ref = state.card_db.clone();
            let db = db_ref.as_ref().expect("card_db required");
            let ctx = super::tokens::build_dynamic_context(state, controller);
            let n = amount.evaluate(
                controller,
                &state.objects,
                &state.battlefield,
                &|id| db.get(id),
                Some(&ctx),
            );
            if n > 0 {
                for target in targets {
                    if let Target::Player(p) = target {
                        let old_life = state.players[*p].life;
                        state.players[*p].life -= n;
                        state.emit_event(GameEvent::LifeChanged {
                            player: *p,
                            old: old_life,
                            new: state.players[*p].life,
                        });
                    }
                }
                // If no explicit targets, apply to controller
                if targets.is_empty() {
                    let old_life = state.players[controller].life;
                    state.players[controller].life -= n;
                    state.emit_event(GameEvent::LifeChanged {
                        player: controller,
                        old: old_life,
                        new: state.players[controller].life,
                    });
                }
            }
        }

        Effect::ExtraTurn => {
            state.extra_turns.push_back(controller);
        }

        Effect::SkipPhase(phase) => {
            state.skip_phases.insert(*phase);
        }

        Effect::Multiple(effects) => {
            for e in effects {
                resolve_effect(state, e, controller, targets);
            }
        }

        Effect::SearchLibrary { destination, subtype_filter } => {
            if !subtype_filter.is_empty() {
                // Fetch-land style: filter by subtype. Always present a choice
                // (or fail to find if no matching cards in library).
                state.pending_tutor = Some(crate::game::PendingTutor {
                    controller,
                    destination: *destination,
                    subtype_filter: subtype_filter.clone(),
                });
            } else if state.players[controller].tutor_targets.is_empty() {
                // Legacy behavior: no tutor targets configured, take top card.
                if !state.players[controller].library.is_empty() {
                    let card_obj = state.players[controller].library.remove(0);
                    state.move_object(card_obj, ZoneType::Library, *destination);
                }
            } else {
                // Set pending tutor — the player will choose via ChooseTutorTarget.
                state.pending_tutor = Some(crate::game::PendingTutor {
                    controller,
                    destination: *destination,
                    subtype_filter: vec![],
                });
            }
        }

        Effect::BounceAllNonlandOpponents => {
            // Bounce all nonland permanents opponents control to their owners' hands.
            let db = state.card_db();
            let to_bounce: Vec<ObjectId> = state
                .battlefield
                .iter()
                .copied()
                .filter(|&id| {
                    if let Some(inst) = state.objects.get(&id) {
                        if inst.controller == controller {
                            return false; // skip own permanents
                        }
                        if let Some(def) = db.get(inst.card_def_id) {
                            return !def.is_land();
                        }
                    }
                    false
                })
                .collect();
            for id in to_bounce {
                state.move_object(id, ZoneType::Battlefield, ZoneType::Hand);
            }
        }

        Effect::ReturnToTopOfLibrary { .. } => {
            // Put target card from graveyard on top of owner's library.
            for target in targets {
                match target {
                    Target::Object(obj_id) => {
                        if let Some(inst) = state.objects.get(obj_id) {
                            let owner = inst.owner;
                            state.move_object(*obj_id, ZoneType::Graveyard, ZoneType::Library);
                            // Move to front (top) of library
                            if let Some(pos) = state.players[owner].library.iter().position(|&id| id == *obj_id) {
                                let id = state.players[owner].library.remove(pos);
                                state.players[owner].library.insert(0, id);
                            }
                        }
                    }
                    _ => {}
                }
            }
        }

        Effect::UntapTarget { .. } => {
            for target in targets {
                match target {
                    Target::Object(obj_id) => {
                        if let Some(inst) = state.objects.get_mut(obj_id) {
                            inst.tapped = false;
                        }
                    }
                    _ => {}
                }
            }
        }

        Effect::Unimplemented(_) => {
            // Can't resolve unimplemented effects
        }
    }
}
