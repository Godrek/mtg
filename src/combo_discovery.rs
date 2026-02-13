//! Combo discovery through exhaustive exploration of card interactions.
//!
//! Instead of relying on manually registered combos or discovering them
//! through full game simulation, this module takes a set of cards and
//! exhaustively explores all possible sequences of ability activations
//! to find cycles that represent infinite combos.
//!
//! # Algorithm
//!
//! For each subset of cards (typically pairs and triples):
//! 1. Place all cards on a virtual "battlefield" (the combo workspace)
//! 2. DFS through all possible ability activations (mana abilities,
//!    activated abilities)
//! 3. Track the "configuration" (which pieces are tapped/untapped)
//!    along the search path
//! 4. When revisiting a configuration with equal or greater resources,
//!    a cycle is detected — this represents an infinite combo
//!
//! Static abilities (e.g., Kinnan's mana bonus) are modeled as modifiers
//! that affect mana production during the exploration.
//!
//! # Example
//!
//! Basalt Monolith + Kinnan, Bonder Prodigy:
//! - Step 1: Tap Monolith for {C}{C}{C}, Kinnan adds {C} → pool = 4
//! - Step 2: Pay {3} to untap Monolith → pool = 1
//! - Cycle detected: same configuration (both untapped), net +1 mana
//!
//! # Usage
//!
//! ```ignore
//! let db = sample::build_sample_db();
//! let card_ids = vec![BASALT_MONOLITH, KINNAN_BONDER_PRODIGY, THRASIOS_TRITON_HERO];
//! let config = DiscoveryConfig::default();
//! let combos = discover_combos(&db, &card_ids, &config);
//! for combo in &combos {
//!     println!("{}", combo);
//! }
//! ```

use std::collections::HashMap;
use std::fmt;

use crate::card::{CardDef, CardId, CardType, Effect, ManaAbility, TargetSpec};
use crate::combo::{ComboDef, ComboEffect, ComboPrecondition, ComboRegistry};
use crate::game::CardDatabase;
use crate::layers::StaticAbility;
use crate::mana::{Color, ManaPool};

// =========================================================================
// Configuration
// =========================================================================

/// Configuration for the combo discovery process.
#[derive(Debug, Clone)]
pub struct DiscoveryConfig {
    /// Maximum number of cards to consider in a single combo (default: 3).
    /// Higher values find more complex combos but increase search time
    /// exponentially: C(N, max_pieces) combinations explored.
    pub max_combo_pieces: usize,

    /// Maximum depth of the action sequence DFS (default: 20).
    /// Longer sequences are unlikely to represent practical combos.
    pub max_depth: usize,

    /// Maximum initial mana to try when kick-starting combos (default: 10).
    /// Some combos require seed mana to begin the loop. We try
    /// starting with 0, 1, ..., max_startup_mana colorless mana.
    pub max_startup_mana: u32,

    /// Minimum net mana per cycle to consider a combo worth registering
    /// (default: 1). Break-even loops (net 0) are not useful.
    pub min_net_mana: u32,

    /// Default reward weight for discovered combos (default: 0.2).
    pub default_reward_weight: f64,
}

impl Default for DiscoveryConfig {
    fn default() -> Self {
        DiscoveryConfig {
            max_combo_pieces: 3,
            max_depth: 20,
            max_startup_mana: 10,
            min_net_mana: 1,
            default_reward_weight: 0.2,
        }
    }
}

// =========================================================================
// Internal state for exploration
// =========================================================================

/// A snapshot of which pieces are tapped — the "configuration" for cycle detection.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
struct PieceConfig {
    tapped: Vec<bool>,
}

/// Resources tracked during exploration (beyond the mana pool).
#[derive(Clone, Debug, Default)]
struct SideEffects {
    damage_dealt: u32,
    life_gained: u32,
    cards_drawn: u32,
}

/// The mini-state used during combo exploration.
#[derive(Clone, Debug)]
struct ExploreState {
    /// Which pieces are tapped.
    tapped: Vec<bool>,
    /// Current mana pool.
    mana_pool: ManaPool,
    /// Accumulated side effects during this exploration path.
    side_effects: SideEffects,
}

impl ExploreState {
    fn config(&self) -> PieceConfig {
        PieceConfig {
            tapped: self.tapped.clone(),
        }
    }
}

/// An action taken during combo exploration.
#[derive(Clone, Debug)]
enum ComboAction {
    /// Activate a mana ability on a piece.
    ActivateManaAbility {
        piece_index: usize,
        ability_index: usize,
    },
    /// Activate a non-mana ability on a piece.
    ActivateAbility {
        piece_index: usize,
        ability_index: usize,
    },
}

/// Contextual information about the static abilities present across all pieces.
struct StaticContext {
    /// How many pieces have ManaFromNonlandBonus (e.g., Kinnan).
    mana_from_nonland_bonus_count: u32,
}

// =========================================================================
// Discovery results
// =========================================================================

/// A combo discovered through exhaustive exploration.
#[derive(Debug, Clone)]
pub struct DiscoveredCombo {
    /// The card IDs involved in this combo.
    pub pieces: Vec<CardId>,
    /// Human-readable card names.
    pub piece_names: Vec<String>,
    /// Net colorless mana gained per cycle.
    pub net_colorless_per_cycle: u32,
    /// Net colored mana gained per cycle (by color).
    pub net_colored_per_cycle: HashMap<Color, u32>,
    /// Damage dealt to opponent per cycle.
    pub damage_per_cycle: u32,
    /// Life gained per cycle.
    pub life_per_cycle: u32,
    /// Cards drawn per cycle.
    pub cards_per_cycle: u32,
    /// The action sequence that forms one cycle of the combo.
    pub cycle_actions: Vec<String>,
    /// Minimum startup mana required to begin the loop.
    pub startup_mana: u32,
}

impl DiscoveredCombo {
    /// Generate a human-readable name for this combo.
    pub fn name(&self) -> String {
        let names = self.piece_names.join(" + ");
        let mut effects = Vec::new();

        let total_mana = self.net_colorless_per_cycle
            + self.net_colored_per_cycle.values().sum::<u32>();
        if total_mana > 0 {
            effects.push("Infinite Mana".to_string());
        }
        if self.damage_per_cycle > 0 {
            effects.push("Infinite Damage".to_string());
        }
        if self.life_per_cycle > 0 {
            effects.push("Infinite Life".to_string());
        }
        if self.cards_per_cycle > 0 {
            effects.push("Infinite Draw".to_string());
        }

        if effects.is_empty() {
            effects.push("Infinite Loop".to_string());
        }

        format!("{} {}", names, effects.join(" + "))
    }

    /// Convert to a ComboDef suitable for registration in the ComboRegistry.
    pub fn to_combo_def(&self, reward_weight: f64) -> ComboDef {
        let effect = self.build_combo_effect();

        ComboDef {
            id: 0, // Assigned by registry.register()
            name: self.name(),
            required_pieces: self.pieces.clone(),
            preconditions: Vec::new(), // Set by discover_and_register()
            effect,
            reward_weight,
        }
    }

    fn build_combo_effect(&self) -> ComboEffect {
        let mut effects = Vec::new();

        if self.net_colorless_per_cycle > 0 {
            // Produce 100 as "effectively infinite"
            effects.push(ComboEffect::AddColorlessMana(100));
        }

        for (&color, &amount) in &self.net_colored_per_cycle {
            if amount > 0 {
                effects.push(ComboEffect::AddColoredMana(color, 100));
            }
        }

        if self.damage_per_cycle > 0 {
            effects.push(ComboEffect::DealDamageToOpponent(100));
        }

        if self.life_per_cycle > 0 {
            effects.push(ComboEffect::GainLife(100));
        }

        if self.cards_per_cycle > 0 {
            effects.push(ComboEffect::DrawCards(50));
        }

        match effects.len() {
            0 => ComboEffect::AddColorlessMana(0),
            1 => effects.into_iter().next().unwrap(),
            _ => ComboEffect::Multiple(effects),
        }
    }
}

impl fmt::Display for DiscoveredCombo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Combo: {}", self.name())?;
        writeln!(f, "  Pieces: {}", self.piece_names.join(", "))?;

        if self.startup_mana > 0 {
            writeln!(f, "  Startup mana: {}", self.startup_mana)?;
        }

        let total_mana = self.net_colorless_per_cycle
            + self.net_colored_per_cycle.values().sum::<u32>();
        if total_mana > 0 {
            writeln!(f, "  Net mana/cycle: +{}", total_mana)?;
        }
        if self.damage_per_cycle > 0 {
            writeln!(f, "  Damage/cycle: {}", self.damage_per_cycle)?;
        }
        if self.life_per_cycle > 0 {
            writeln!(f, "  Life/cycle: +{}", self.life_per_cycle)?;
        }
        if self.cards_per_cycle > 0 {
            writeln!(f, "  Draw/cycle: {}", self.cards_per_cycle)?;
        }

        writeln!(f, "  Cycle ({} steps):", self.cycle_actions.len())?;
        for (i, action) in self.cycle_actions.iter().enumerate() {
            writeln!(f, "    {}. {}", i + 1, action)?;
        }

        Ok(())
    }
}

// =========================================================================
// Main discovery function
// =========================================================================

/// Discover infinite combos among a set of cards.
///
/// Examines all combinations of cards (up to `config.max_combo_pieces` at a
/// time) and exhaustively explores ability activation sequences to find cycles
/// that produce net-positive resources.
///
/// Returns a list of discovered combos, sorted by piece count (simpler first).
pub fn discover_combos(
    db: &CardDatabase,
    card_ids: &[CardId],
    config: &DiscoveryConfig,
) -> Vec<DiscoveredCombo> {
    let mut discovered = Vec::new();
    let mut seen_piece_sets: Vec<Vec<CardId>> = Vec::new();

    // Try all subsets of increasing size
    for size in 2..=config.max_combo_pieces.min(card_ids.len()) {
        for_each_combination(card_ids, size, |subset| {
            // Skip if a subset of these pieces already forms a combo
            // (e.g., if A+B is a combo, don't report A+B+C)
            let dominated = seen_piece_sets.iter().any(|known| {
                known.iter().all(|id| subset.contains(id))
            });
            if dominated {
                return;
            }

            if let Some(combo) = explore_combination(db, subset, config) {
                seen_piece_sets.push(combo.pieces.clone());
                discovered.push(combo);
            }
        });
    }

    discovered
}

/// Discover combos and automatically register them in a ComboRegistry.
///
/// This is the primary entry point: discover combos among the given cards,
/// then register each as a macro-action in the returned registry.
pub fn discover_and_register(
    db: &CardDatabase,
    card_ids: &[CardId],
    config: &DiscoveryConfig,
) -> (ComboRegistry, Vec<DiscoveredCombo>) {
    let combos = discover_combos(db, card_ids, config);
    let mut registry = ComboRegistry::new();

    for combo in &combos {
        let mut combo_def = combo.to_combo_def(config.default_reward_weight);

        // Set preconditions: pieces with mana abilities that require tap
        // should be untapped for the combo to activate
        for &piece_id in &combo.pieces {
            if let Some(def) = db.get(piece_id) {
                if !def.mana_abilities.is_empty() {
                    combo_def
                        .preconditions
                        .push(ComboPrecondition::PieceUntapped(piece_id));
                }
            }
        }

        registry.register(combo_def);
    }

    (registry, combos)
}

// =========================================================================
// Exploration engine
// =========================================================================

/// Explore a specific combination of cards for infinite combos.
fn explore_combination(
    db: &CardDatabase,
    piece_ids: &[CardId],
    config: &DiscoveryConfig,
) -> Option<DiscoveredCombo> {
    // Look up card definitions
    let pieces: Vec<&CardDef> = piece_ids.iter().filter_map(|id| db.get(*id)).collect();

    if pieces.len() != piece_ids.len() {
        return None; // Some cards not found in DB
    }

    // Build static context (check for ability modifiers across all pieces)
    let static_ctx = build_static_context(&pieces);

    // Global visited set: maps each configuration to the best total mana
    // we've explored from. If we reach a config with <= mana, skip it —
    // any cycle reachable from less mana is also reachable from more mana
    // (since all actions available with less mana are also available with
    // more). This prevents combinatorial explosion in non-productive
    // card combinations.
    let mut visited: HashMap<PieceConfig, u32> = HashMap::new();

    // Try with increasing startup mana
    for startup_mana in 0..=config.max_startup_mana {
        let state = ExploreState {
            tapped: vec![false; pieces.len()],
            mana_pool: ManaPool {
                colorless: startup_mana,
                ..ManaPool::empty()
            },
            side_effects: SideEffects::default(),
        };

        let mut path: Vec<(PieceConfig, ManaPool, SideEffects, ComboAction)> = Vec::new();

        if let Some(result) = dfs(
            &state,
            &mut path,
            &pieces,
            &static_ctx,
            config,
            0,
            startup_mana,
            &mut visited,
        ) {
            return Some(result);
        }
    }

    None
}

/// Depth-first search for cycles in the ability activation graph.
///
/// The key insight: with N pieces there are only 2^N possible tap
/// configurations, so cycles are found quickly. We track the path of
/// (config, mana_pool, side_effects) at each step. When we revisit
/// a configuration with >= mana, a net-positive cycle is detected.
///
/// The `visited` map prunes redundant exploration: once we've explored
/// a configuration with N total mana, revisiting it with <= N mana from
/// a different path can't discover any new cycles.
fn dfs(
    state: &ExploreState,
    path: &mut Vec<(PieceConfig, ManaPool, SideEffects, ComboAction)>,
    pieces: &[&CardDef],
    static_ctx: &StaticContext,
    config: &DiscoveryConfig,
    depth: usize,
    startup_mana: u32,
    visited: &mut HashMap<PieceConfig, u32>,
) -> Option<DiscoveredCombo> {
    if depth > config.max_depth {
        return None;
    }

    let current_config = state.config();

    // 1. Check for cycles in the current path (BEFORE visited pruning,
    //    since the cycle itself is what we're looking for).
    for (i, (prev_config, prev_mana, prev_effects, _)) in path.iter().enumerate() {
        if current_config == *prev_config {
            // Same configuration — check if we have more resources
            if mana_pool_ge(&state.mana_pool, prev_mana) {
                let net_colorless = state
                    .mana_pool
                    .colorless
                    .saturating_sub(prev_mana.colorless);
                let net_total = mana_pool_diff_total(&state.mana_pool, prev_mana);
                let net_damage = state
                    .side_effects
                    .damage_dealt
                    .saturating_sub(prev_effects.damage_dealt);
                let net_life = state
                    .side_effects
                    .life_gained
                    .saturating_sub(prev_effects.life_gained);
                let net_draw = state
                    .side_effects
                    .cards_drawn
                    .saturating_sub(prev_effects.cards_drawn);

                // Must produce something useful
                let produces_something =
                    net_total >= config.min_net_mana || net_damage > 0 || net_life > 0 || net_draw > 0;

                if produces_something {
                    // Build the cycle action descriptions from the path
                    let cycle_actions: Vec<String> = path[i..]
                        .iter()
                        .map(|(_, _, _, action)| describe_action(action, pieces))
                        .collect();

                    // Build net colored mana map
                    let mut net_colored = HashMap::new();
                    for &color in &Color::ALL {
                        let prev = prev_mana.get(color);
                        let curr = state.mana_pool.get(color);
                        if curr > prev {
                            net_colored.insert(color, curr - prev);
                        }
                    }

                    return Some(DiscoveredCombo {
                        pieces: pieces.iter().map(|p| p.id).collect(),
                        piece_names: pieces.iter().map(|p| p.name.clone()).collect(),
                        net_colorless_per_cycle: net_colorless,
                        net_colored_per_cycle: net_colored,
                        damage_per_cycle: net_damage,
                        life_per_cycle: net_life,
                        cards_per_cycle: net_draw,
                        cycle_actions,
                        startup_mana,
                    });
                }
            }
        }
    }

    // 2. Visited pruning: if we've already fully explored this config
    //    with >= total mana, no new cycles can be found from here.
    let total_mana = state.mana_pool.total();
    if let Some(&best_mana) = visited.get(&current_config) {
        if total_mana <= best_mana {
            return None;
        }
    }
    visited.insert(current_config.clone(), total_mana);

    // 3. Generate all legal actions and explore each
    let actions = legal_combo_actions(state, pieces);

    for action in actions {
        let mut new_state = state.clone();
        apply_combo_action(&mut new_state, &action, pieces, static_ctx);

        path.push((
            current_config.clone(),
            state.mana_pool.clone(),
            state.side_effects.clone(),
            action,
        ));

        let result = dfs(
            &new_state,
            path,
            pieces,
            static_ctx,
            config,
            depth + 1,
            startup_mana,
            visited,
        );

        if result.is_some() {
            path.pop();
            return result;
        }

        path.pop();
    }

    None
}

// =========================================================================
// Action generation and application
// =========================================================================

/// Generate all legal actions in the current combo exploration state.
fn legal_combo_actions(state: &ExploreState, pieces: &[&CardDef]) -> Vec<ComboAction> {
    let mut actions = Vec::new();

    for (piece_idx, piece) in pieces.iter().enumerate() {
        // Mana abilities (require the piece to be untapped for tap abilities)
        if !state.tapped[piece_idx] {
            for (ability_idx, _ma) in piece.mana_abilities.iter().enumerate() {
                actions.push(ComboAction::ActivateManaAbility {
                    piece_index: piece_idx,
                    ability_index: ability_idx,
                });
            }
        }

        // Activated abilities
        for (ability_idx, ability) in piece.activated_abilities.iter().enumerate() {
            // Check tap requirement
            if ability.requires_tap && state.tapped[piece_idx] {
                continue;
            }

            // Check mana cost
            if !state.mana_pool.can_pay(&ability.cost) {
                continue;
            }

            // Only consider abilities with effects we can model in combo discovery
            if is_combo_relevant_effect(&ability.effect) {
                actions.push(ComboAction::ActivateAbility {
                    piece_index: piece_idx,
                    ability_index: ability_idx,
                });
            }
        }
    }

    actions
}

/// Check if an effect is relevant for combo discovery.
///
/// We only model effects that directly contribute to resource cycles:
/// untapping, mana production, damage, life, and card draw.
fn is_combo_relevant_effect(effect: &Effect) -> bool {
    match effect {
        Effect::UntapTarget { .. }
        | Effect::AddMana { .. }
        | Effect::DealDamage { .. }
        | Effect::GainLife { .. }
        | Effect::DrawCards { .. }
        | Effect::LoseLife { .. } => true,
        Effect::Multiple(effects) => effects.iter().any(is_combo_relevant_effect),
        _ => false,
    }
}

/// Apply a combo action to the exploration state.
fn apply_combo_action(
    state: &mut ExploreState,
    action: &ComboAction,
    pieces: &[&CardDef],
    static_ctx: &StaticContext,
) {
    match action {
        ComboAction::ActivateManaAbility {
            piece_index,
            ability_index,
        } => {
            let piece = pieces[*piece_index];
            let ma = &piece.mana_abilities[*ability_index];

            // Produce mana
            match ma {
                ManaAbility::TapForColor(color) => {
                    state.mana_pool.add_color(*color, 1);
                }
                ManaAbility::TapForColorless => {
                    state.mana_pool.colorless += 1;
                }
                ManaAbility::TapForAny => {
                    // For discovery purposes, treat as colorless
                    state.mana_pool.colorless += 1;
                }
                ManaAbility::TapForChoice(_colors) => {
                    // For discovery purposes, treat as colorless
                    state.mana_pool.colorless += 1;
                }
                ManaAbility::TapForColorlessAmount(n) => {
                    state.mana_pool.colorless += n;
                }
            }

            // Apply ManaFromNonlandBonus (e.g., Kinnan) if the source is nonland
            if static_ctx.mana_from_nonland_bonus_count > 0
                && !piece.card_types.contains(&CardType::Land)
            {
                state.mana_pool.colorless += static_ctx.mana_from_nonland_bonus_count;
            }

            // Tap the piece
            state.tapped[*piece_index] = true;
        }

        ComboAction::ActivateAbility {
            piece_index,
            ability_index,
        } => {
            let piece = pieces[*piece_index];
            let ability = &piece.activated_abilities[*ability_index];

            // Pay mana cost
            state.mana_pool.pay(&ability.cost);

            // Tap if required
            if ability.requires_tap {
                state.tapped[*piece_index] = true;
            }

            // Apply effect
            apply_combo_effect(state, &ability.effect, *piece_index, pieces);
        }
    }
}

/// Apply an effect during combo exploration.
fn apply_combo_effect(
    state: &mut ExploreState,
    effect: &Effect,
    source_piece: usize,
    pieces: &[&CardDef],
) {
    match effect {
        Effect::UntapTarget { target } => {
            match target {
                // Controller target on an activated ability = untap self
                // (e.g., Basalt Monolith: "{3}: Untap Basalt Monolith")
                TargetSpec::Controller => {
                    state.tapped[source_piece] = false;
                }
                // For other untap targets, try untapping each tapped piece
                _ => {
                    for i in 0..pieces.len() {
                        if i != source_piece && state.tapped[i] {
                            state.tapped[i] = false;
                            break;
                        }
                    }
                }
            }
        }
        Effect::AddMana { color, amount } => match color {
            Some(c) => state.mana_pool.add_color(*c, *amount),
            None => state.mana_pool.colorless += amount,
        },
        Effect::DealDamage { amount, target } => {
            if matches!(target, TargetSpec::Opponent | TargetSpec::AnyPlayer) {
                state.side_effects.damage_dealt += amount;
            }
        }
        Effect::GainLife { amount } => {
            state.side_effects.life_gained += amount;
        }
        Effect::DrawCards { count } => {
            state.side_effects.cards_drawn += count;
        }
        Effect::LoseLife { amount, target } => {
            if matches!(target, TargetSpec::Opponent) {
                state.side_effects.damage_dealt += amount;
            }
        }
        Effect::Multiple(effects) => {
            for sub in effects {
                apply_combo_effect(state, sub, source_piece, pieces);
            }
        }
        _ => {} // Other effects not modeled in combo discovery
    }
}

// =========================================================================
// Helpers
// =========================================================================

/// Build the static context from the pieces on the battlefield.
fn build_static_context(pieces: &[&CardDef]) -> StaticContext {
    let mut bonus_count = 0u32;
    for piece in pieces {
        for sa in &piece.static_abilities {
            if matches!(sa, StaticAbility::ManaFromNonlandBonus) {
                bonus_count += 1;
            }
        }
    }
    StaticContext {
        mana_from_nonland_bonus_count: bonus_count,
    }
}

/// Check if mana pool `a` is >= mana pool `b` in every component.
fn mana_pool_ge(a: &ManaPool, b: &ManaPool) -> bool {
    a.white >= b.white
        && a.blue >= b.blue
        && a.black >= b.black
        && a.red >= b.red
        && a.green >= b.green
        && a.colorless >= b.colorless
}

/// Total mana difference (a - b), clamped to 0.
fn mana_pool_diff_total(a: &ManaPool, b: &ManaPool) -> u32 {
    a.total().saturating_sub(b.total())
}

/// Generate a human-readable description of a combo action.
fn describe_action(action: &ComboAction, pieces: &[&CardDef]) -> String {
    match action {
        ComboAction::ActivateManaAbility {
            piece_index,
            ability_index,
        } => {
            let piece = pieces[*piece_index];
            let ma = &piece.mana_abilities[*ability_index];
            let mana_desc = match ma {
                ManaAbility::TapForColor(c) => format!("{{{}}}", color_symbol(*c)),
                ManaAbility::TapForColorless => "{C}".to_string(),
                ManaAbility::TapForAny => "{any}".to_string(),
                ManaAbility::TapForChoice(colors) => {
                    let syms: Vec<String> =
                        colors.iter().map(|c| color_symbol(*c).to_string()).collect();
                    format!("{{{}}}", syms.join("/"))
                }
                ManaAbility::TapForColorlessAmount(n) => {
                    format!("{{{C}}}", C = "C".repeat(*n as usize))
                }
            };
            format!("Tap {} for {}", piece.name, mana_desc)
        }
        ComboAction::ActivateAbility {
            piece_index,
            ability_index,
        } => {
            let piece = pieces[*piece_index];
            let ability = &piece.activated_abilities[*ability_index];
            format!("Activate {}: {}", piece.name, ability.description)
        }
    }
}

fn color_symbol(color: Color) -> char {
    match color {
        Color::White => 'W',
        Color::Blue => 'U',
        Color::Black => 'B',
        Color::Red => 'R',
        Color::Green => 'G',
    }
}

/// Iterate over all combinations of `size` elements from `items`.
fn for_each_combination<T: Clone>(items: &[T], size: usize, mut callback: impl FnMut(&[T])) {
    if size == 0 || size > items.len() {
        return;
    }
    let mut indices: Vec<usize> = (0..size).collect();
    let mut buffer: Vec<T> = indices.iter().map(|&i| items[i].clone()).collect();

    loop {
        callback(&buffer);

        // Find the rightmost index that can be incremented
        let mut i = size;
        loop {
            if i == 0 {
                return; // All combinations exhausted
            }
            i -= 1;
            if indices[i] < items.len() - size + i {
                break;
            }
        }

        // Increment it and reset all indices to the right
        indices[i] += 1;
        for j in (i + 1)..size {
            indices[j] = indices[j - 1] + 1;
        }

        // Update buffer
        for (k, &idx) in indices.iter().enumerate() {
            buffer[k] = items[idx].clone();
        }
    }
}

// =========================================================================
// Tests
// =========================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::card::sample::{self, ids};

    #[test]
    fn test_discover_basalt_kinnan() {
        let db = sample::build_sample_db();
        let cards = vec![ids::BASALT_MONOLITH, ids::KINNAN_BONDER_PRODIGY];
        let config = DiscoveryConfig::default();

        let combos = discover_combos(&db, &cards, &config);
        assert!(
            !combos.is_empty(),
            "Should discover Basalt Monolith + Kinnan combo"
        );

        let combo = &combos[0];
        assert!(combo.pieces.contains(&ids::BASALT_MONOLITH));
        assert!(combo.pieces.contains(&ids::KINNAN_BONDER_PRODIGY));
        assert!(
            combo.net_colorless_per_cycle >= 1,
            "Net mana per cycle should be >= 1, got {}",
            combo.net_colorless_per_cycle
        );
        assert_eq!(
            combo.startup_mana, 0,
            "Basalt + Kinnan needs 0 startup mana"
        );
    }

    #[test]
    fn test_basalt_alone_is_not_infinite() {
        let db = sample::build_sample_db();
        // Basalt Monolith alone: taps for 3, pay 3 to untap = break-even
        let cards = vec![ids::BASALT_MONOLITH];
        let config = DiscoveryConfig {
            max_combo_pieces: 2,
            ..Default::default()
        };

        let combos = discover_combos(&db, &cards, &config);
        assert!(
            combos.is_empty(),
            "Basalt Monolith alone should not be an infinite combo"
        );
    }

    #[test]
    fn test_grim_kinnan_is_not_infinite() {
        let db = sample::build_sample_db();
        // Grim Monolith + Kinnan: taps for 3, +1 = 4, but costs 4 to untap = break-even
        let cards = vec![ids::GRIM_MONOLITH, ids::KINNAN_BONDER_PRODIGY];
        let config = DiscoveryConfig::default();

        let combos = discover_combos(&db, &cards, &config);
        assert!(
            combos.is_empty(),
            "Grim Monolith + Kinnan should be break-even, not infinite"
        );
    }

    #[test]
    fn test_discover_from_larger_pool() {
        let db = sample::build_sample_db();
        let cards = vec![
            ids::BASALT_MONOLITH,
            ids::KINNAN_BONDER_PRODIGY,
            ids::THRASIOS_TRITON_HERO,
            ids::GRIM_MONOLITH,
            ids::SOL_RING,
        ];
        let config = DiscoveryConfig::default();

        let combos = discover_combos(&db, &cards, &config);

        // Should find Basalt + Kinnan as a 2-piece combo
        let basalt_kinnan = combos.iter().find(|c| {
            c.pieces.contains(&ids::BASALT_MONOLITH)
                && c.pieces.contains(&ids::KINNAN_BONDER_PRODIGY)
                && c.pieces.len() == 2
        });
        assert!(
            basalt_kinnan.is_some(),
            "Should find Basalt + Kinnan in larger pool"
        );
    }

    #[test]
    fn test_combo_display() {
        let db = sample::build_sample_db();
        let cards = vec![ids::BASALT_MONOLITH, ids::KINNAN_BONDER_PRODIGY];
        let config = DiscoveryConfig::default();

        let combos = discover_combos(&db, &cards, &config);
        assert!(!combos.is_empty());

        let display = format!("{}", combos[0]);
        assert!(display.contains("Basalt Monolith"));
        assert!(display.contains("Kinnan"));
    }

    #[test]
    fn test_discover_and_register() {
        let db = sample::build_sample_db();
        let cards = vec![ids::BASALT_MONOLITH, ids::KINNAN_BONDER_PRODIGY];
        let config = DiscoveryConfig::default();

        let (registry, combos) = discover_and_register(&db, &cards, &config);
        assert!(!combos.is_empty());
        assert!(!registry.combos.is_empty());

        let combo_def = registry.get(0).unwrap();
        assert!(combo_def.required_pieces.contains(&ids::BASALT_MONOLITH));
        assert!(
            combo_def
                .required_pieces
                .contains(&ids::KINNAN_BONDER_PRODIGY)
        );
    }

    #[test]
    fn test_combination_iterator() {
        let items = vec![1, 2, 3, 4];
        let mut results = Vec::new();
        for_each_combination(&items, 2, |combo| {
            results.push(combo.to_vec());
        });
        assert_eq!(results.len(), 6); // C(4,2) = 6
        assert!(results.contains(&vec![1, 2]));
        assert!(results.contains(&vec![3, 4]));
    }

    #[test]
    fn test_combination_single() {
        let items = vec![10, 20, 30];
        let mut results = Vec::new();
        for_each_combination(&items, 1, |combo| {
            results.push(combo.to_vec());
        });
        assert_eq!(results.len(), 3);
    }

    #[test]
    fn test_combination_full_set() {
        let items = vec![1, 2, 3];
        let mut results = Vec::new();
        for_each_combination(&items, 3, |combo| {
            results.push(combo.to_vec());
        });
        assert_eq!(results.len(), 1); // C(3,3) = 1
        assert_eq!(results[0], vec![1, 2, 3]);
    }

    #[test]
    fn test_mana_pool_ge() {
        let a = ManaPool {
            colorless: 5,
            ..ManaPool::empty()
        };
        let b = ManaPool {
            colorless: 3,
            ..ManaPool::empty()
        };
        assert!(mana_pool_ge(&a, &b));
        assert!(!mana_pool_ge(&b, &a));

        // Equal counts as >=
        let c = ManaPool {
            colorless: 5,
            ..ManaPool::empty()
        };
        assert!(mana_pool_ge(&a, &c));
    }

    #[test]
    fn test_no_combos_among_vanilla_creatures() {
        let db = sample::build_sample_db();
        let cards = vec![
            ids::GRIZZLY_BEARS,
            ids::GREY_OGRE,
            ids::SERRA_ANGEL,
        ];
        let config = DiscoveryConfig::default();

        let combos = discover_combos(&db, &cards, &config);
        assert!(
            combos.is_empty(),
            "Vanilla creatures should not produce combos"
        );
    }

    #[test]
    fn test_superset_pruning() {
        let db = sample::build_sample_db();
        // If Basalt+Kinnan is found as a 2-piece combo, the 3-piece
        // combination Basalt+Kinnan+Thrasios should be pruned as redundant.
        let cards = vec![
            ids::BASALT_MONOLITH,
            ids::KINNAN_BONDER_PRODIGY,
            ids::THRASIOS_TRITON_HERO,
        ];
        let config = DiscoveryConfig::default();

        let combos = discover_combos(&db, &cards, &config);

        // Should find exactly 1 combo (the 2-piece version)
        assert_eq!(
            combos.len(),
            1,
            "Should find exactly the 2-piece combo, not supersets"
        );
        assert_eq!(combos[0].pieces.len(), 2);
    }
}
