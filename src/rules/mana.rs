use crate::card::{ManaAbility, ObjectId};
use crate::game::{GameState, PlayerIndex};

/// Compute the total generic cost reduction for a spell being cast by `player`.
/// Checks all permanents the player controls for `CostReduction` abilities.
pub fn total_cost_reduction(state: &GameState, player: PlayerIndex, is_creature: bool) -> u32 {
    use crate::card::CostReductionTarget;
    let db = state.card_db();
    let mut total = 0u32;
    for &obj_id in &state.battlefield {
        let inst = &state.objects[&obj_id];
        if inst.controller != player {
            continue;
        }
        let def = match db.get(inst.card_def_id) {
            Some(d) => d,
            None => continue,
        };
        if let Some(ref reduction) = def.cost_reduction {
            let applies = match reduction.applies_to {
                CostReductionTarget::AllSpells => true,
                CostReductionTarget::CreatureSpells => is_creature,
            };
            if applies {
                total += reduction.generic_reduction;
            }
        }
    }
    total
}

/// Apply cost reduction to a ManaCost, returning the reduced cost.
pub fn apply_cost_reduction(cost: &crate::mana::ManaCost, reduction: u32) -> crate::mana::ManaCost {
    let mut reduced = cost.clone();
    reduced.generic = reduced.generic.saturating_sub(reduction);
    reduced
}

/// A tap decision: which land to tap and what mana it produces.
enum TapDecision {
    Color(ObjectId, crate::mana::Color),
    Colorless(ObjectId, u32),
}

/// Auto-tap lands to pay a mana cost.
/// Simple greedy: tap colored sources first for colored requirements, then any for generic.
/// Uses a two-phase approach: collect tap decisions (read-only), then apply them (mutate).
pub fn auto_tap_lands(
    state: &mut GameState,
    player: PlayerIndex,
    cost: &crate::mana::ManaCost,
) {
    use crate::mana::Color;

    let mut decisions: Vec<TapDecision> = Vec::new();

    // Phase 1: Collect tap decisions (immutable borrow)
    {
        let db = state.card_db();
        let untapped = state.untapped_lands(player);
        let mut tapped_set = std::collections::HashSet::new();

        // Pay colored costs first
        for &color in &Color::ALL {
            let needed = cost.color_amount(color);
            let already_have = state.players[player].mana_pool.get(color);
            if needed <= already_have {
                continue;
            }
            let mut still_need = needed - already_have;

            for &land_id in &untapped {
                if still_need == 0 {
                    break;
                }
                if tapped_set.contains(&land_id) {
                    continue;
                }
                let inst = &state.objects[&land_id];
                if inst.tapped {
                    continue;
                }
                let def = match db.get(inst.card_def_id) {
                    Some(d) => d,
                    None => continue,
                };

                for ma in &def.mana_abilities {
                    let produces_color = match ma {
                        ManaAbility::TapForColor(c) => *c == color,
                        ManaAbility::TapForChoice(colors) => colors.contains(&color),
                        ManaAbility::TapForAny => true,
                        _ => false,
                    };
                    if produces_color {
                        decisions.push(TapDecision::Color(land_id, color));
                        tapped_set.insert(land_id);
                        still_need -= 1;
                        break;
                    }
                }
            }
        }

        // Pay generic costs from remaining untapped lands
        let colored_from_decisions = decisions.len() as u32;
        let pool_total = state.players[player].mana_pool.total() + colored_from_decisions;
        let colored_total: u32 = Color::ALL.iter().map(|&c| cost.color_amount(c)).sum();
        if pool_total < colored_total + cost.generic {
            let mut still_need = (colored_total + cost.generic).saturating_sub(pool_total);

            for &land_id in &untapped {
                if still_need == 0 {
                    break;
                }
                if tapped_set.contains(&land_id) {
                    continue;
                }
                let inst = &state.objects[&land_id];
                if inst.tapped {
                    continue;
                }
                let def = match db.get(inst.card_def_id) {
                    Some(d) => d,
                    None => continue,
                };

                if let Some(ma) = def.mana_abilities.first() {
                    let produced = match ma {
                        ManaAbility::TapForColor(c) => {
                            decisions.push(TapDecision::Color(land_id, *c));
                            1
                        }
                        ManaAbility::TapForColorless
                        | ManaAbility::TapForAny => {
                            decisions.push(TapDecision::Colorless(land_id, 1));
                            1
                        }
                        ManaAbility::TapForColorlessAmount(n) => {
                            decisions.push(TapDecision::Colorless(land_id, *n));
                            *n
                        }
                        ManaAbility::TapForChoice(colors) => {
                            if let Some(&c) = colors.first() {
                                decisions.push(TapDecision::Color(land_id, c));
                            } else {
                                decisions.push(TapDecision::Colorless(land_id, 1));
                            }
                            1
                        }
                    };
                    tapped_set.insert(land_id);
                    still_need = still_need.saturating_sub(produced);
                }
            }
        }
    }

    // Phase 2: Apply decisions (mutable borrow)
    for decision in decisions {
        match decision {
            TapDecision::Color(land_id, color) => {
                state.players[player].mana_pool.add_color(color, 1);
                if let Some(inst) = state.objects.get_mut(&land_id) {
                    inst.tapped = true;
                }
            }
            TapDecision::Colorless(land_id, amount) => {
                state.players[player].mana_pool.colorless += amount;
                if let Some(inst) = state.objects.get_mut(&land_id) {
                    inst.tapped = true;
                }
            }
        }
    }
}
