use rand::seq::SliceRandom;

use crate::action::{legal_actions, Action};
use crate::card::ObjectId;
use crate::game::{GameState, PlayerIndex};

/// A strategy decides what action to take given a game state.
/// This is the interface the GTO solver will optimize over.
pub trait Strategy: Send + Sync {
    fn choose_action(&self, state: &GameState, player: PlayerIndex) -> Action;
    fn name(&self) -> &str;
}

/// Random strategy: picks uniformly at random from legal actions.
/// Useful as a baseline and for Monte Carlo rollouts.
pub struct RandomStrategy;

impl Strategy for RandomStrategy {
    fn choose_action(&self, state: &GameState, _player: PlayerIndex) -> Action {
        let mut rng = rand::thread_rng();
        let actions = legal_actions(state);
        actions
            .choose(&mut rng)
            .cloned()
            .unwrap_or(Action::PassPriority)
    }

    fn name(&self) -> &str {
        "Random"
    }
}

/// Greedy heuristic strategy: plays lands, casts biggest spell possible,
/// attacks with everything, blocks favorably.
/// Better than random, serves as a reasonable default opponent.
pub struct GreedyStrategy;

impl Strategy for GreedyStrategy {
    fn choose_action(&self, state: &GameState, _player: PlayerIndex) -> Action {
        let actions = legal_actions(state);
        let db = state.card_db();

        // Priority 0: If we must order triggers or replacement effects, pick
        // the first ordering (FIFO). A real MCCFR solver would evaluate all
        // orderings; greedy just uses FIFO.
        for action in &actions {
            if matches!(action, Action::OrderTriggers { .. } | Action::ChooseReplacementOrder { .. }) {
                return action.clone();
            }
        }

        // Priority 1: Play a land if we can
        for action in &actions {
            if let Action::PlayLand { .. } = action {
                return action.clone();
            }
        }

        // Priority 2: Cast the most expensive spell we can afford
        let mut best_spell: Option<&Action> = None;
        let mut best_cmc = 0;
        for action in &actions {
            if let Action::CastSpell { object_id, .. } = action {
                let inst = &state.objects[object_id];
                if let Some(def) = db.get(inst.card_def_id) {
                    let cmc = def.cmc();
                    if cmc >= best_cmc {
                        best_cmc = cmc;
                        best_spell = Some(action);
                    }
                }
            }
        }
        if let Some(spell) = best_spell {
            return spell.clone();
        }

        // Priority 3: Attack with all eligible creatures
        for action in &actions {
            if let Action::DeclareAttackers { attackers } = action {
                if !attackers.is_empty() {
                    // Find the action that attacks with the most creatures
                    let mut best_attack: Option<&Action> = None;
                    let mut max_attackers = 0;
                    for a in &actions {
                        if let Action::DeclareAttackers { attackers } = a {
                            if attackers.len() > max_attackers {
                                max_attackers = attackers.len();
                                best_attack = Some(a);
                            }
                        }
                    }
                    if let Some(attack) = best_attack {
                        return attack.clone();
                    }
                }
            }
        }

        // Priority 4: Block with favorable trades
        for action in &actions {
            if let Action::DeclareBlockers { blocks } = action {
                if !blocks.is_empty() {
                    // Simple heuristic: block if our creature's toughness > attacker's power
                    // (i.e., we survive the block)
                    let mut best_block: Option<&Action> = None;
                    let mut best_score: i32 = 0;

                    for a in &actions {
                        if let Action::DeclareBlockers { blocks } = a {
                            let score = evaluate_blocks(state, blocks);
                            if score > best_score || best_block.is_none() {
                                best_score = score;
                                best_block = Some(a);
                            }
                        }
                    }
                    if best_score > 0 {
                        if let Some(block) = best_block {
                            return block.clone();
                        }
                    }
                }
            }
        }

        // Priority 5: Cleanup discard if forced
        for action in &actions {
            if let Action::Discard { .. } = action {
                return action.clone();
            }
        }

        // Default: pass priority
        Action::PassPriority
    }

    fn name(&self) -> &str {
        "Greedy"
    }
}

/// Evaluate a blocking assignment. Positive = good for the defender.
fn evaluate_blocks(state: &GameState, blocks: &[(ObjectId, ObjectId)]) -> i32 {
    let db = state.card_db();
    let mut score: i32 = 0;

    for &(blocker_id, attacker_id) in blocks {
        let blocker_inst = &state.objects[&blocker_id];
        let attacker_inst = &state.objects[&attacker_id];

        let blocker_def = match db.get(blocker_inst.card_def_id) {
            Some(d) => d,
            None => continue,
        };
        let attacker_def = match db.get(attacker_inst.card_def_id) {
            Some(d) => d,
            None => continue,
        };

        let attacker_power = attacker_inst.effective_power(attacker_def);
        let attacker_toughness = attacker_inst.effective_toughness(attacker_def);
        let blocker_power = blocker_inst.effective_power(blocker_def);
        let blocker_toughness = blocker_inst.effective_toughness(blocker_def);

        let blocker_dies = attacker_power >= blocker_toughness;
        let attacker_dies = blocker_power >= attacker_toughness;

        // Prevented damage (the attacker's power that won't hit the player)
        score += attacker_power;

        if attacker_dies && !blocker_dies {
            // We kill their creature and ours survives — great trade
            score += attacker_def.cmc() as i32 * 2;
        } else if attacker_dies && blocker_dies {
            // Trade — good if their creature is more expensive
            score += (attacker_def.cmc() as i32) - (blocker_def.cmc() as i32);
        } else if !attacker_dies && blocker_dies {
            // We lose our creature, they keep theirs — bad unless preventing lethal
            score -= blocker_def.cmc() as i32;
        }
    }

    score
}
