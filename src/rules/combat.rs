use crate::card::{KeywordAbility, ObjectId};
use crate::events::GameEvent;
use crate::game::{GameState, PlayerIndex, Target};

/// Check if any creature in combat has first strike or double strike.
pub(super) fn has_first_strike_creatures(state: &GameState) -> bool {
    let check = |id: &ObjectId| -> bool {
        state.has_keyword(*id, KeywordAbility::FirstStrike)
            || state.has_keyword(*id, KeywordAbility::DoubleStrike)
    };

    state.combat.attackers.iter().any(check)
        || state.combat.blockers.keys().any(check)
}

/// A pending damage application collected during combat resolution.
struct DamageEvent {
    /// The attacker/source dealing the damage.
    source_id: ObjectId,
    target_object: Option<ObjectId>,
    target_player: Option<PlayerIndex>,
    amount: u32,
    lifelink_for: Option<PlayerIndex>,
}

/// Resolve combat damage. Collects all damage events first (read phase), then applies them.
pub(super) fn resolve_combat_damage(state: &mut GameState, first_strike_only: bool) {
    let defending_player = state.opponent(state.active_player);
    let mut damage_events: Vec<DamageEvent> = Vec::new();

    // Phase 1: Collect all damage events (immutable — uses layer engine)
    {
        let attackers = state.combat.attackers.clone();

        for &attacker_id in &attackers {
            if state.objects.get(&attacker_id).is_none() {
                continue;
            }

            let has_fs = state.has_keyword(attacker_id, KeywordAbility::FirstStrike);
            let has_ds = state.has_keyword(attacker_id, KeywordAbility::DoubleStrike);

            let deals_damage = if first_strike_only {
                has_fs || has_ds
            } else {
                !has_fs || has_ds
            };

            if !deals_damage {
                continue;
            }

            let power = state.effective_power(attacker_id).max(0) as u32;
            if power == 0 {
                continue;
            }

            let lifelink = if state.has_keyword(attacker_id, KeywordAbility::Lifelink) {
                Some(state.objects[&attacker_id].controller)
            } else {
                None
            };

            let blockers = state
                .combat
                .attacker_blockers
                .get(&attacker_id)
                .cloned()
                .unwrap_or_default();

            if blockers.is_empty() {
                // Unblocked — damage to defending player
                damage_events.push(DamageEvent {
                    source_id: attacker_id,
                    target_object: None,
                    target_player: Some(defending_player),
                    amount: power,
                    lifelink_for: lifelink,
                });
            } else {
                let has_trample = state.has_keyword(attacker_id, KeywordAbility::Trample);
                let has_deathtouch = state.has_keyword(attacker_id, KeywordAbility::Deathtouch);

                let mut remaining = power;

                for &blocker_id in &blockers {
                    if remaining == 0 {
                        break;
                    }
                    let blocker_inst = match state.objects.get(&blocker_id) {
                        Some(i) => i,
                        None => continue,
                    };

                    let eff_toughness = state.effective_toughness(blocker_id);
                    let remaining_tough =
                        (eff_toughness - blocker_inst.damage_marked as i32).max(0) as u32;
                    let damage = if has_deathtouch {
                        1.min(remaining)
                    } else {
                        remaining_tough.min(remaining)
                    };

                    damage_events.push(DamageEvent {
                        source_id: attacker_id,
                        target_object: Some(blocker_id),
                        target_player: None,
                        amount: damage,
                        lifelink_for: lifelink,
                    });
                    remaining -= damage;
                }

                // Trample: excess to defending player
                if has_trample && remaining > 0 {
                    damage_events.push(DamageEvent {
                        source_id: attacker_id,
                        target_object: None,
                        target_player: Some(defending_player),
                        amount: remaining,
                        lifelink_for: lifelink,
                    });
                }

                // Blockers deal damage to attacker
                for &blocker_id in &blockers {
                    if state.objects.get(&blocker_id).is_none() {
                        continue;
                    }

                    let blocker_has_fs =
                        state.has_keyword(blocker_id, KeywordAbility::FirstStrike);
                    let blocker_has_ds =
                        state.has_keyword(blocker_id, KeywordAbility::DoubleStrike);

                    let blocker_deals = if first_strike_only {
                        blocker_has_fs || blocker_has_ds
                    } else {
                        !blocker_has_fs || blocker_has_ds
                    };

                    if blocker_deals {
                        let blocker_power =
                            state.effective_power(blocker_id).max(0) as u32;
                        let blocker_lifelink =
                            if state.has_keyword(blocker_id, KeywordAbility::Lifelink) {
                                Some(state.objects[&blocker_id].controller)
                            } else {
                                None
                            };

                        damage_events.push(DamageEvent {
                            source_id: blocker_id,
                            target_object: Some(attacker_id),
                            target_player: None,
                            amount: blocker_power,
                            lifelink_for: blocker_lifelink,
                        });
                    }
                }
            }
        }
    }

    // Phase 2: Apply all damage events (mutable)
    for event in damage_events {
        if let Some(obj_id) = event.target_object {
            if let Some(inst) = state.objects.get_mut(&obj_id) {
                inst.damage_marked += event.amount;
            }
            state.emit_event(GameEvent::DamageDealt {
                source: event.source_id,
                target: Target::Object(obj_id),
                amount: event.amount,
                is_combat: true,
            });
        }
        if let Some(player) = event.target_player {
            let old_life = state.players[player].life;
            state.players[player].life -= event.amount as i32;

            // Commander damage tracking (CR 903.10a)
            if state.is_commander(event.source_id) {
                let source_owner = state.objects.get(&event.source_id)
                    .map(|i| i.owner)
                    .unwrap_or(0);
                if player < state.players.len()
                    && source_owner < state.players[player].commander_damage_received.len()
                {
                    state.players[player].commander_damage_received[source_owner] +=
                        event.amount as i32;
                }
            }

            state.emit_event(GameEvent::DamageDealt {
                source: event.source_id,
                target: Target::Player(player),
                amount: event.amount,
                is_combat: true,
            });
            state.emit_event(GameEvent::LifeChanged {
                player,
                old: old_life,
                new: state.players[player].life,
            });
        }
        if let Some(lifelink_player) = event.lifelink_for {
            let old_life = state.players[lifelink_player].life;
            state.players[lifelink_player].life += event.amount as i32;
            state.emit_event(GameEvent::LifeChanged {
                player: lifelink_player,
                old: old_life,
                new: state.players[lifelink_player].life,
            });
        }
    }
}
