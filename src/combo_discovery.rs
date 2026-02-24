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
//!    activated abilities) with triggered ability chains
//! 3. Track the "configuration" (which pieces are tapped/untapped)
//!    along the search path
//! 4. When revisiting a configuration with equal or greater resources,
//!    a cycle is detected — this represents an infinite combo
//!
//! The engine models:
//! - **Mana abilities** (tap for mana) with static modifiers (Kinnan)
//! - **Activated abilities** with mana costs, tap costs, and sacrifice costs
//! - **Triggered abilities** that fire when creatures die (Thornbite Staff,
//!   Blood Artist, Ogre Slumlord)
//! - **Creature tokens** as a tracked resource (Marrow-Gnawer creates rats)
//! - **Equipment links** (Thornbite Staff untaps equipped creature)
//!
//! # Examples
//!
//! Basalt Monolith + Kinnan, Bonder Prodigy:
//! - Tap Monolith for {C}{C}{C}, Kinnan adds {C} → pool = 4
//! - Pay {3} to untap Monolith → pool = 1
//! - Cycle detected: same config, net +1 mana
//!
//! Marrow-Gnawer + Thornbite Staff (with 2+ other rats):
//! - Tap Marrow-Gnawer, sacrifice a rat → rat dies
//! - Death trigger: Thornbite Staff untaps Marrow-Gnawer
//! - Effect: create X rat tokens (X = rats controlled)
//! - Cycle detected: same config, net +N creatures
//!
//! # Usage
//!
//! ```ignore
//! let db = sample::build_sample_db();
//! let card_ids = vec![BASALT_MONOLITH, KINNAN_BONDER_PRODIGY];
//! let config = DiscoveryConfig::default();
//! let combos = discover_combos(&db, &card_ids, &config);
//! for combo in &combos {
//!     println!("{}", combo);
//! }
//! ```

use std::collections::HashMap;
use std::fmt;

use crate::card::{
    CardDef, CardId, CardType, DynamicValue, Effect, ManaAbility, SacrificeCost, TargetSpec,
    TokenDef, TriggerCondition,
};
use crate::combo::{
    ComboDef, ComboCategory, ComboPrecondition, ComboRegistry,
    categorize, effect_from_categories,
};
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
    pub max_combo_pieces: usize,

    /// Maximum depth of the action sequence DFS (default: 20).
    pub max_depth: usize,

    /// Maximum initial mana to try when kick-starting combos (default: 10).
    pub max_startup_mana: u32,

    /// Minimum net mana per cycle to consider a mana-only combo worth
    /// registering (default: 1). Does not apply when damage/life/draw/tokens
    /// are produced.
    pub min_net_mana: u32,

    /// Default reward weight for discovered combos (default: 0.2).
    pub default_reward_weight: f64,

    /// Maximum initial creature tokens to try (default: 5).
    /// Some combos require seed creatures (e.g., Marrow-Gnawer needs rats).
    pub max_startup_creatures: u32,

    /// Maximum initial artifact tokens to try (default: 5).
    /// Some combos require seed artifacts (e.g., Clock of Omens needs tappable artifacts).
    pub max_startup_artifacts: u32,
}

impl Default for DiscoveryConfig {
    fn default() -> Self {
        DiscoveryConfig {
            max_combo_pieces: 3,
            max_depth: 20,
            max_startup_mana: 10,
            min_net_mana: 1,
            default_reward_weight: 0.2,
            max_startup_creatures: 5,
            max_startup_artifacts: 5,
        }
    }
}

// =========================================================================
// Internal state for exploration
// =========================================================================

/// A snapshot of which pieces are tapped — the "configuration" for cycle
/// detection. Resources (mana, creature tokens) are tracked separately
/// since they grow/shrink and we detect cycles by comparing resource
/// levels across visits to the same configuration.
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
    /// Number of creature tokens on the battlefield.
    /// Includes all creatures that can be sacrificed (non-piece creatures).
    creature_tokens: u32,
    /// Number of untapped artifact tokens on the battlefield.
    /// Used to pay tap-artifact costs (e.g., Clock of Omens) and
    /// artifact sacrifice costs (e.g., Krark-Clan Ironworks).
    artifact_tokens: u32,
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
        /// For TapForAny/TapForChoice abilities, which color to produce.
        /// None means produce colorless.
        color_choice: Option<Color>,
    },
    /// Activate a non-mana ability on a piece (may involve sacrifice).
    ActivateAbility {
        piece_index: usize,
        ability_index: usize,
        /// For abilities with targeted untap effects (non-Controller),
        /// which piece to untap. None means the ability doesn't have
        /// a targeted untap, or targets self (Controller).
        untap_target: Option<usize>,
        /// Piece indices tapped to pay a tap-artifacts cost (e.g., Clock of Omens).
        /// The remaining cost (tap_artifacts_cost - pieces.len()) is paid from artifact_tokens.
        tapped_pieces_for_cost: Vec<usize>,
    },
}

/// Contextual information about static abilities and equipment links.
struct StaticContext {
    /// How many pieces have ManaFromNonlandBonus (e.g., Kinnan).
    mana_from_nonland_bonus_count: u32,
    /// Equipment links: (equipment_piece_idx, equipped_creature_idx).
    /// When an equipment's triggered ability fires with "untap equipped
    /// creature" (UntapTarget { target: Controller }), we untap the
    /// linked creature instead of the equipment itself.
    equipment_links: Vec<(usize, usize)>,
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
    /// What kind(s) of infinite loop this is.
    pub categories: Vec<ComboCategory>,
    /// Net colorless mana gained per cycle.
    pub net_colorless_per_cycle: u32,
    /// Net colored mana gained per cycle (by color).
    pub net_colored_per_cycle: HashMap<Color, u32>,
    /// Net creature tokens gained per cycle.
    pub net_creatures_per_cycle: u32,
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
    /// Minimum startup creature tokens required.
    pub startup_creatures: u32,
    /// Minimum startup artifact tokens required.
    pub startup_artifacts: u32,
}

impl DiscoveredCombo {
    /// Generate a human-readable name for this combo.
    pub fn name(&self) -> String {
        let names = self.piece_names.join(" + ");
        let effects: Vec<&str> = self.categories.iter().map(|c| match c {
            ComboCategory::InfiniteMana => "Infinite Mana",
            ComboCategory::InfiniteTokens => "Infinite Tokens",
            ComboCategory::InfiniteDamage => "Infinite Damage",
            ComboCategory::InfiniteLifeGain => "Infinite Life",
            ComboCategory::InfiniteDraw => "Infinite Draw",
        }).collect();

        if effects.is_empty() {
            format!("{} Infinite Loop", names)
        } else {
            format!("{} {}", names, effects.join(" + "))
        }
    }

    /// Convert to a ComboDef suitable for registration in the ComboRegistry.
    pub fn to_combo_def(&self, reward_weight: f64) -> ComboDef {
        let effect = effect_from_categories(&self.categories);

        ComboDef {
            id: 0, // Assigned by registry.register()
            name: self.name(),
            categories: self.categories.clone(),
            required_pieces: self.pieces.clone(),
            preconditions: Vec::new(), // Set by discover_and_register()
            effect,
            reward_weight,
        }
    }
}

impl fmt::Display for DiscoveredCombo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Combo: {}", self.name())?;
        writeln!(f, "  Pieces: {}", self.piece_names.join(", "))?;

        let cat_names: Vec<&str> = self.categories.iter().map(|c| match c {
            ComboCategory::InfiniteMana => "Infinite Mana",
            ComboCategory::InfiniteTokens => "Infinite Tokens",
            ComboCategory::InfiniteDamage => "Infinite Damage",
            ComboCategory::InfiniteLifeGain => "Infinite Life Gain",
            ComboCategory::InfiniteDraw => "Infinite Draw",
        }).collect();
        writeln!(f, "  Categories: {}", cat_names.join(", "))?;

        if self.startup_mana > 0 {
            writeln!(f, "  Startup mana: {}", self.startup_mana)?;
        }
        if self.startup_creatures > 0 {
            writeln!(f, "  Startup creatures: {}", self.startup_creatures)?;
        }
        if self.startup_artifacts > 0 {
            writeln!(f, "  Startup artifacts: {}", self.startup_artifacts)?;
        }

        let total_mana = self.net_colorless_per_cycle
            + self.net_colored_per_cycle.values().sum::<u32>();
        if total_mana > 0 {
            writeln!(f, "  Net mana/cycle: +{}", total_mana)?;
        }
        if self.net_creatures_per_cycle > 0 {
            writeln!(f, "  Net creatures/cycle: +{}", self.net_creatures_per_cycle)?;
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

    // Try all subsets of increasing size.
    // We do NOT prune supersets: a 3-piece combo can be meaningfully
    // different from a 2-piece subset (e.g., adding Blood Artist to
    // Marrow-Gnawer + Thornbite Staff produces infinite damage with
    // fewer startup creatures).
    for size in 2..=config.max_combo_pieces.min(card_ids.len()) {
        for_each_combination(card_ids, size, |subset| {
            if let Some(combo) = explore_combination(db, subset, config) {
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

        // Set preconditions: pieces with mana abilities or tap-requiring
        // activated abilities should be untapped for the combo to activate
        for &piece_id in &combo.pieces {
            if let Some(def) = db.get(piece_id) {
                let has_mana_abilities = !def.mana_abilities.is_empty();
                let has_tap_ability = def
                    .activated_abilities
                    .iter()
                    .any(|a| a.requires_tap);
                if has_mana_abilities || has_tap_ability {
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

/// Check if a piece is an Equipment (has Equipment subtype).
fn is_equipment(piece: &CardDef) -> bool {
    piece
        .subtypes
        .iter()
        .any(|s| s.0 == "Equipment")
}

/// Check if a piece is a creature.
fn is_creature(piece: &CardDef) -> bool {
    piece.card_types.contains(&CardType::Creature)
}

/// Check if a piece is an artifact.
fn is_artifact(piece: &CardDef) -> bool {
    piece.card_types.contains(&CardType::Artifact)
}

/// Check if any piece has a sacrifice-cost ability or a death trigger,
/// meaning creature tokens are relevant for this combination.
fn needs_creature_tokens(pieces: &[&CardDef]) -> bool {
    for piece in pieces {
        for ability in &piece.activated_abilities {
            if let Some(ref sac) = ability.sacrifice_cost {
                if matches!(sac, SacrificeCost::AnyCreature | SacrificeCost::CreatureWithSubtype(_)) {
                    return true;
                }
            }
        }
    }
    false
}

/// Check if any piece has a tap-artifacts cost, artifact sacrifice cost,
/// or creates artifact tokens, meaning artifact tokens are relevant.
fn needs_artifact_tokens(pieces: &[&CardDef]) -> bool {
    for piece in pieces {
        for ability in &piece.activated_abilities {
            if ability.tap_artifacts_cost.is_some() {
                return true;
            }
            if let Some(SacrificeCost::Artifacts(_)) = ability.sacrifice_cost {
                return true;
            }
        }
    }
    false
}

/// Explore a specific combination of cards for infinite combos.
fn explore_combination(
    db: &CardDatabase,
    piece_ids: &[CardId],
    config: &DiscoveryConfig,
) -> Option<DiscoveredCombo> {
    let pieces: Vec<&CardDef> = piece_ids.iter().filter_map(|id| db.get(*id)).collect();

    if pieces.len() != piece_ids.len() {
        return None;
    }

    // Determine which equipment configurations to try.
    // Equipment pieces can be attached to any creature piece.
    let equipment_indices: Vec<usize> = pieces
        .iter()
        .enumerate()
        .filter(|(_, p)| is_equipment(p))
        .map(|(i, _)| i)
        .collect();
    let creature_indices: Vec<usize> = pieces
        .iter()
        .enumerate()
        .filter(|(_, p)| is_creature(p))
        .map(|(i, _)| i)
        .collect();

    // Generate all possible equipment attachment configurations.
    // Each equipment can be attached to any creature, or unattached.
    let attachment_configs = if equipment_indices.is_empty() || creature_indices.is_empty() {
        vec![vec![]] // No equipment links
    } else {
        generate_attachment_configs(&equipment_indices, &creature_indices)
    };

    // Determine if creature/artifact tokens are relevant for this combination
    let uses_creatures = needs_creature_tokens(&pieces);
    let uses_artifacts = needs_artifact_tokens(&pieces);

    // Try each equipment configuration
    for links in &attachment_configs {
        let static_ctx = StaticContext {
            mana_from_nonland_bonus_count: count_mana_bonus(&pieces),
            equipment_links: links.clone(),
        };

        // Determine creature and artifact token ranges to try
        let max_creatures = if uses_creatures {
            config.max_startup_creatures
        } else {
            0
        };
        let max_artifacts = if uses_artifacts {
            config.max_startup_artifacts
        } else {
            0
        };

        let mut visited: HashMap<PieceConfig, (u32, u32, u32)> = HashMap::new();

        // Invariant: iterating from 0 upward and returning on the first
        // combo found guarantees that the reported startup_mana,
        // startup_creatures, and startup_artifacts are the minimum values required.
        for startup_artifacts in 0..=max_artifacts {
            for startup_creatures in 0..=max_creatures {
                for startup_mana in 0..=config.max_startup_mana {
                    let state = ExploreState {
                        tapped: vec![false; pieces.len()],
                        mana_pool: ManaPool {
                            colorless: startup_mana,
                            ..ManaPool::empty()
                        },
                        creature_tokens: startup_creatures,
                        artifact_tokens: startup_artifacts,
                        side_effects: SideEffects::default(),
                    };

                    let mut path: Vec<(PieceConfig, ManaPool, u32, u32, SideEffects, ComboAction)> =
                        Vec::new();

                    if let Some(result) = dfs(
                        &state,
                        &mut path,
                        &pieces,
                        &static_ctx,
                        config,
                        0,
                        startup_mana,
                        startup_creatures,
                        startup_artifacts,
                        &mut visited,
                    ) {
                        return Some(result);
                    }
                }
            }
        }
    }

    None
}

/// Generate all equipment attachment configurations.
///
/// Each equipment piece can be attached to any creature piece.
/// Returns a list of link vectors, where each link is (equipment_idx, creature_idx).
fn generate_attachment_configs(
    equipment_indices: &[usize],
    creature_indices: &[usize],
) -> Vec<Vec<(usize, usize)>> {
    let mut configs = Vec::new();

    if equipment_indices.len() == 1 {
        // Single equipment: try attaching to each creature
        for &ci in creature_indices {
            configs.push(vec![(equipment_indices[0], ci)]);
        }
    } else {
        // Multiple equipment: try all combinations
        // For simplicity, enumerate all possible assignments
        let mut current = vec![0usize; equipment_indices.len()];
        loop {
            let links: Vec<(usize, usize)> = equipment_indices
                .iter()
                .zip(current.iter())
                .map(|(&ei, &ci_idx)| (ei, creature_indices[ci_idx]))
                .collect();
            configs.push(links);

            // Increment: find rightmost index that can be bumped
            let mut carry = true;
            for idx in (0..current.len()).rev() {
                if carry {
                    current[idx] += 1;
                    if current[idx] >= creature_indices.len() {
                        current[idx] = 0;
                    } else {
                        carry = false;
                    }
                }
            }
            if carry {
                break; // All combinations exhausted
            }
        }
    }

    configs
}

/// Depth-first search for cycles in the ability activation graph.
///
/// The `visited` map prunes redundant exploration: once we've explored
/// a configuration with N total mana, M creatures, and K artifacts,
/// revisiting with <= mana, <= creatures, and <= artifacts can't discover new cycles.
fn dfs(
    state: &ExploreState,
    path: &mut Vec<(PieceConfig, ManaPool, u32, u32, SideEffects, ComboAction)>,
    pieces: &[&CardDef],
    static_ctx: &StaticContext,
    config: &DiscoveryConfig,
    depth: usize,
    startup_mana: u32,
    startup_creatures: u32,
    startup_artifacts: u32,
    visited: &mut HashMap<PieceConfig, (u32, u32, u32)>,
) -> Option<DiscoveredCombo> {
    if depth > config.max_depth {
        return None;
    }

    let current_config = state.config();

    // 1. Check for cycles in the current path.
    for (i, (prev_config, prev_mana, prev_creatures, prev_artifacts, prev_effects, _)) in
        path.iter().enumerate()
    {
        if current_config == *prev_config {
            if mana_pool_ge(&state.mana_pool, prev_mana)
                && state.creature_tokens >= *prev_creatures
                && state.artifact_tokens >= *prev_artifacts
            {
                let net_colorless = state
                    .mana_pool
                    .colorless
                    .saturating_sub(prev_mana.colorless);
                let net_total_mana = mana_pool_diff_total(&state.mana_pool, prev_mana);
                let net_creatures = state.creature_tokens.saturating_sub(*prev_creatures);
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

                let produces_something = net_total_mana >= config.min_net_mana
                    || net_creatures > 0
                    || net_damage > 0
                    || net_life > 0
                    || net_draw > 0;

                if produces_something {
                    let cycle_actions: Vec<String> = path[i..]
                        .iter()
                        .map(|(_, _, _, _, _, action)| describe_action(action, pieces))
                        .collect();

                    let mut net_colored = HashMap::new();
                    for &color in &Color::ALL {
                        let prev = prev_mana.get(color);
                        let curr = state.mana_pool.get(color);
                        if curr > prev {
                            net_colored.insert(color, curr - prev);
                        }
                    }

                    let net_colored_total = net_colored.values().sum::<u32>();
                    let categories = categorize(
                        net_colorless,
                        net_colored_total,
                        net_creatures,
                        net_damage,
                        net_life,
                        net_draw,
                    );

                    return Some(DiscoveredCombo {
                        pieces: pieces.iter().map(|p| p.id).collect(),
                        piece_names: pieces.iter().map(|p| p.name.clone()).collect(),
                        categories,
                        net_colorless_per_cycle: net_colorless,
                        net_colored_per_cycle: net_colored,
                        net_creatures_per_cycle: net_creatures,
                        damage_per_cycle: net_damage,
                        life_per_cycle: net_life,
                        cards_per_cycle: net_draw,
                        cycle_actions,
                        startup_mana,
                        startup_creatures,
                        startup_artifacts,
                    });
                }
            }
        }
    }

    // 2. Visited pruning: skip if we've explored this config with
    //    >= mana AND >= creatures AND >= artifacts.
    let total_mana = state.mana_pool.total();
    let creatures = state.creature_tokens;
    let artifacts = state.artifact_tokens;
    if let Some(&(best_mana, best_creatures, best_artifacts)) = visited.get(&current_config) {
        if total_mana <= best_mana && creatures <= best_creatures && artifacts <= best_artifacts {
            return None;
        }
    }
    visited.insert(current_config.clone(), (total_mana, creatures, artifacts));

    // 3. Generate all legal actions and explore each
    let actions = legal_combo_actions(state, pieces);

    for action in actions {
        let mut new_state = state.clone();
        apply_combo_action(&mut new_state, &action, pieces, static_ctx);

        path.push((
            current_config.clone(),
            state.mana_pool.clone(),
            state.creature_tokens,
            state.artifact_tokens,
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
            startup_creatures,
            startup_artifacts,
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

    // Collect colors needed by activated ability costs in this combo,
    // so TapForAny only branches on relevant colors (not all 5).
    let needed_colors = colors_needed_by_pieces(pieces);

    for (piece_idx, piece) in pieces.iter().enumerate() {
        // Mana abilities (require the piece to be untapped for tap abilities)
        if !state.tapped[piece_idx] {
            for (ability_idx, ma) in piece.mana_abilities.iter().enumerate() {
                match ma {
                    ManaAbility::TapForAny => {
                        // Generate one action per needed color, plus colorless.
                        actions.push(ComboAction::ActivateManaAbility {
                            piece_index: piece_idx,
                            ability_index: ability_idx,
                            color_choice: None,
                        });
                        for &color in &needed_colors {
                            actions.push(ComboAction::ActivateManaAbility {
                                piece_index: piece_idx,
                                ability_index: ability_idx,
                                color_choice: Some(color),
                            });
                        }
                    }
                    ManaAbility::TapForChoice(colors) => {
                        // Generate one action per listed color option.
                        for &color in colors {
                            actions.push(ComboAction::ActivateManaAbility {
                                piece_index: piece_idx,
                                ability_index: ability_idx,
                                color_choice: Some(color),
                            });
                        }
                    }
                    _ => {
                        // Fixed-output abilities: no choice needed.
                        actions.push(ComboAction::ActivateManaAbility {
                            piece_index: piece_idx,
                            ability_index: ability_idx,
                            color_choice: None,
                        });
                    }
                }
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

            // Check sacrifice cost
            if let Some(ref sac_cost) = ability.sacrifice_cost {
                if !can_pay_sacrifice(state, sac_cost) {
                    continue;
                }
            }

            // Only consider abilities with effects we can model
            if !is_combo_relevant_effect(&ability.effect) {
                continue;
            }

            // Determine tap-artifact cost options (branching axis 1).
            // Each option is a list of piece indices to tap + number of tokens to consume.
            let tap_cost_options: Vec<Vec<usize>> = if let Some(cost) = ability.tap_artifacts_cost {
                // Identify untapped artifact pieces available for tapping.
                // Exclude the source piece if it's also being tapped (requires_tap).
                let candidate_pieces: Vec<usize> = (0..pieces.len())
                    .filter(|&i| {
                        !state.tapped[i]
                            && is_artifact(pieces[i])
                            && !(ability.requires_tap && i == piece_idx)
                    })
                    .collect();

                let mut options = Vec::new();
                // Enumerate: k pieces tapped + (cost - k) tokens consumed
                for k in 0..=std::cmp::min(cost as usize, candidate_pieces.len()) {
                    let tokens_needed = cost - k as u32;
                    if tokens_needed > state.artifact_tokens {
                        continue;
                    }
                    // Generate all k-element subsets of candidate_pieces
                    for_each_combination_indices(&candidate_pieces, k, |subset| {
                        options.push(subset.to_vec());
                    });
                }

                if options.is_empty() {
                    continue; // Can't pay tap-artifact cost
                }
                options
            } else {
                vec![vec![]] // No tap cost — single option with empty tapped set
            };

            // For each tap-cost option, determine valid untap targets.
            // Untap targets depend on which pieces will be tapped:
            // pieces already tapped + pieces tapped by requires_tap + pieces
            // tapped for the artifact cost are all valid targets.
            for tapped_pieces in &tap_cost_options {
                let untap_targets: Vec<Option<usize>> = if has_targeted_untap(&ability.effect) {
                    let mut targets: Vec<Option<usize>> = Vec::new();
                    for i in 0..pieces.len() {
                        let will_be_tapped = state.tapped[i]
                            || tapped_pieces.contains(&i)
                            || (ability.requires_tap && i == piece_idx);
                        if will_be_tapped {
                            targets.push(Some(i));
                        }
                    }
                    if targets.is_empty() {
                        vec![None]
                    } else {
                        targets
                    }
                } else {
                    vec![None]
                };

                for &untap_target in &untap_targets {
                    actions.push(ComboAction::ActivateAbility {
                        piece_index: piece_idx,
                        ability_index: ability_idx,
                        untap_target,
                        tapped_pieces_for_cost: tapped_pieces.clone(),
                    });
                }
            }
        }
    }

    actions
}

/// Check if the current state can pay a sacrifice cost.
fn can_pay_sacrifice(state: &ExploreState, sac_cost: &SacrificeCost) -> bool {
    match sac_cost {
        SacrificeCost::AnyCreature | SacrificeCost::CreatureWithSubtype(_) => {
            // Creatures are fungible for combo discovery: we intentionally
            // ignore subtype restrictions (e.g., Marrow-Gnawer's "sacrifice
            // a Rat") because the engine identifies *potential* combos —
            // actual legality (does a Rat exist to sacrifice?) is checked at
            // runtime. Tracking token subtypes would require exhaustive
            // search over all possible board states, which is the game
            // engine's job, not the discovery engine's.
            state.creature_tokens >= 1
        }
        SacrificeCost::Artifacts(n) => {
            state.artifact_tokens >= *n
        }
    }
}

/// Check if an effect contains a targeted (non-Controller) UntapTarget.
/// These require branching in the DFS to try each possible untap target.
fn has_targeted_untap(effect: &Effect) -> bool {
    match effect {
        Effect::UntapTarget { target } => !matches!(target, TargetSpec::Controller),
        Effect::Multiple(effects) => effects.iter().any(has_targeted_untap),
        _ => false,
    }
}

/// Check if an effect is relevant for combo discovery.
fn is_combo_relevant_effect(effect: &Effect) -> bool {
    match effect {
        Effect::UntapTarget { .. }
        | Effect::AddMana { .. }
        | Effect::DealDamage { .. }
        | Effect::GainLife { .. }
        | Effect::DrawCards { .. }
        | Effect::LoseLife { .. }
        | Effect::CreateToken(_)
        | Effect::CreateTokens { .. } => true,
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
            color_choice,
        } => {
            let piece = pieces[*piece_index];
            let ma = &piece.mana_abilities[*ability_index];

            match ma {
                ManaAbility::TapForColor(color) => {
                    state.mana_pool.add_color(*color, 1);
                }
                ManaAbility::TapForColorless => {
                    state.mana_pool.colorless += 1;
                }
                ManaAbility::TapForAny | ManaAbility::TapForChoice(_) => {
                    // Use the chosen color, or colorless if no choice.
                    match color_choice {
                        Some(c) => state.mana_pool.add_color(*c, 1),
                        None => state.mana_pool.colorless += 1,
                    }
                }
                ManaAbility::TapForColorlessAmount(n) => {
                    state.mana_pool.colorless += n;
                }
            }

            // ManaFromNonlandBonus (e.g., Kinnan)
            if static_ctx.mana_from_nonland_bonus_count > 0
                && !piece.card_types.contains(&CardType::Land)
            {
                state.mana_pool.colorless += static_ctx.mana_from_nonland_bonus_count;
            }

            state.tapped[*piece_index] = true;
            // Fire tap triggers (e.g., Magda: "Whenever a Dwarf becomes tapped")
            fire_tap_triggers(state, pieces, static_ctx, *piece_index);
        }

        ComboAction::ActivateAbility {
            piece_index,
            ability_index,
            untap_target,
            tapped_pieces_for_cost,
        } => {
            let piece = pieces[*piece_index];
            let ability = &piece.activated_abilities[*ability_index];

            // Pay mana cost
            state.mana_pool.pay(&ability.cost);

            // Tap if required
            if ability.requires_tap {
                state.tapped[*piece_index] = true;
                fire_tap_triggers(state, pieces, static_ctx, *piece_index);
            }

            // Pay tap-artifacts cost: tap specified pieces and consume tokens
            if let Some(cost) = ability.tap_artifacts_cost {
                for &idx in tapped_pieces_for_cost {
                    state.tapped[idx] = true;
                    fire_tap_triggers(state, pieces, static_ctx, idx);
                }
                let tokens_to_consume = cost - tapped_pieces_for_cost.len() as u32;
                state.artifact_tokens = state.artifact_tokens.saturating_sub(tokens_to_consume);
            }

            // Pay sacrifice cost — this happens BEFORE the effect resolves.
            if let Some(ref sac_cost) = ability.sacrifice_cost {
                match sac_cost {
                    SacrificeCost::AnyCreature | SacrificeCost::CreatureWithSubtype(_) => {
                        state.creature_tokens = state.creature_tokens.saturating_sub(1);
                        fire_death_triggers(state, pieces, static_ctx);
                    }
                    SacrificeCost::Artifacts(n) => {
                        state.artifact_tokens = state.artifact_tokens.saturating_sub(*n);
                    }
                }
            }

            // Apply the ability's effect. Any newly created tokens
            // fire ETB triggers (e.g., Ayara drains per entering creature).
            let tokens_before = state.creature_tokens;
            apply_combo_effect(state, &ability.effect, *piece_index, pieces, *untap_target);
            let tokens_created = state.creature_tokens.saturating_sub(tokens_before);
            if tokens_created > 0 {
                fire_etb_triggers(state, pieces, static_ctx, tokens_created);
            }
        }
    }
}

/// Fire all death triggers across all pieces.
///
/// Called when a creature dies (sacrificed, destroyed). Checks each piece
/// for triggered abilities with `ACreatureDies` and applies their effects.
/// After each trigger resolves, any newly created tokens fire ETB triggers.
fn fire_death_triggers(
    state: &mut ExploreState,
    pieces: &[&CardDef],
    static_ctx: &StaticContext,
) {
    for (piece_idx, piece) in pieces.iter().enumerate() {
        for trigger in &piece.triggered_abilities {
            if trigger.trigger == TriggerCondition::ACreatureDies {
                let tokens_before = state.creature_tokens;
                apply_trigger_effect(state, &trigger.effect, piece_idx, pieces, static_ctx);
                let tokens_created = state.creature_tokens.saturating_sub(tokens_before);
                if tokens_created > 0 {
                    fire_etb_triggers(state, pieces, static_ctx, tokens_created);
                }
            }
        }
    }
}

/// Fire all enter-the-battlefield triggers across all pieces.
///
/// Called when creature tokens are created. Each entering creature fires
/// `ACreatureEnters` triggers (e.g., Ayara drains 1 per entering creature).
/// Does not recursively fire ETB triggers for tokens created by ETB effects
/// to avoid infinite loops.
fn fire_etb_triggers(
    state: &mut ExploreState,
    pieces: &[&CardDef],
    static_ctx: &StaticContext,
    count: u32,
) {
    for _ in 0..count {
        for (piece_idx, piece) in pieces.iter().enumerate() {
            for trigger in &piece.triggered_abilities {
                if trigger.trigger == TriggerCondition::ACreatureEnters {
                    apply_trigger_effect(state, &trigger.effect, piece_idx, pieces, static_ctx);
                }
            }
        }
    }
}

/// Fire tap triggers when a piece becomes tapped.
///
/// Checks all pieces for `APermanentWithSubtypeBecomesTapped` triggers and
/// fires them if the tapped piece has the matching subtype (e.g., Magda
/// creates a Treasure when a Dwarf becomes tapped).
fn fire_tap_triggers(
    state: &mut ExploreState,
    pieces: &[&CardDef],
    static_ctx: &StaticContext,
    tapped_piece_idx: usize,
) {
    let tapped_piece = pieces[tapped_piece_idx];
    for (piece_idx, piece) in pieces.iter().enumerate() {
        for trigger in &piece.triggered_abilities {
            if let TriggerCondition::APermanentWithSubtypeBecomesTapped(ref subtype) = trigger.trigger {
                if tapped_piece.subtypes.contains(subtype) {
                    let creatures_before = state.creature_tokens;
                    apply_trigger_effect(state, &trigger.effect, piece_idx, pieces, static_ctx);
                    let creatures_created = state.creature_tokens.saturating_sub(creatures_before);
                    if creatures_created > 0 {
                        fire_etb_triggers(state, pieces, static_ctx, creatures_created);
                    }
                }
            }
        }
    }
}

/// Apply a triggered ability effect.
///
/// For equipment pieces, `UntapTarget { target: Controller }` means
/// "untap equipped creature" — we resolve this through the equipment links
/// in the static context.
fn apply_trigger_effect(
    state: &mut ExploreState,
    effect: &Effect,
    source_piece: usize,
    pieces: &[&CardDef],
    static_ctx: &StaticContext,
) {
    match effect {
        Effect::UntapTarget { target } => {
            if matches!(target, TargetSpec::Controller) {
                // Check if this is an equipment piece with a link
                if let Some(&(_, equipped_idx)) = static_ctx
                    .equipment_links
                    .iter()
                    .find(|&&(eq_idx, _)| eq_idx == source_piece)
                {
                    // Untap the equipped creature
                    state.tapped[equipped_idx] = false;
                } else {
                    // Not equipment — untap self
                    state.tapped[source_piece] = false;
                }
            }
        }
        Effect::CreateToken(token) => {
            apply_token_creation(state, token, 1);
        }
        Effect::CreateTokens { token, count } => {
            let n = evaluate_dynamic_value(count, state, pieces);
            apply_token_creation(state, token, n);
        }
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
                apply_trigger_effect(state, sub, source_piece, pieces, static_ctx);
            }
        }
        _ => {}
    }
}

/// Apply an activated ability effect during combo exploration.
///
/// `untap_target` specifies which piece to untap for non-Controller
/// UntapTarget effects. This is chosen during action generation so
/// the DFS branches on all valid targets.
fn apply_combo_effect(
    state: &mut ExploreState,
    effect: &Effect,
    source_piece: usize,
    pieces: &[&CardDef],
    untap_target: Option<usize>,
) {
    match effect {
        Effect::UntapTarget { target } => {
            match target {
                TargetSpec::Controller => {
                    state.tapped[source_piece] = false;
                }
                _ => {
                    if let Some(idx) = untap_target {
                        state.tapped[idx] = false;
                    }
                    // If untap_target is None, nothing is tapped to untap
                }
            }
        }
        Effect::AddMana { color, amount } => match color {
            Some(c) => state.mana_pool.add_color(*c, *amount),
            None => state.mana_pool.colorless += amount,
        },
        Effect::CreateToken(token) => {
            apply_token_creation(state, token, 1);
        }
        Effect::CreateTokens { token, count } => {
            let n = evaluate_dynamic_value(count, state, pieces);
            apply_token_creation(state, token, n);
        }
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
                apply_combo_effect(state, sub, source_piece, pieces, untap_target);
            }
        }
        _ => {}
    }
}

/// Route token creation to the appropriate counter based on card types.
///
/// Artifact tokens (e.g., Treasure) go to artifact_tokens.
/// Creature tokens (e.g., Rats, Dragons) go to creature_tokens.
/// Tokens with no card_types default to creature_tokens for backward compatibility.
fn apply_token_creation(state: &mut ExploreState, token: &TokenDef, count: u32) {
    let is_artifact = token.card_types.contains(&CardType::Artifact);
    let is_creature = token.card_types.contains(&CardType::Creature) || token.card_types.is_empty();
    if is_artifact {
        state.artifact_tokens += count;
    }
    if is_creature {
        state.creature_tokens += count;
    }
}

/// Evaluate a DynamicValue in the combo discovery context.
fn evaluate_dynamic_value(
    value: &DynamicValue,
    state: &ExploreState,
    pieces: &[&CardDef],
) -> u32 {
    match value {
        DynamicValue::CreaturesControlled => {
            // Count: creature tokens + creature pieces on the battlefield
            let piece_creatures = pieces
                .iter()
                .filter(|p| is_creature(p))
                .count() as u32;
            state.creature_tokens + piece_creatures
        }
        // Other dynamic values not relevant for combo discovery
        _ => 1,
    }
}

// =========================================================================
// Helpers
// =========================================================================

/// Collect the set of colors required by activated ability costs across all
/// pieces. Used to limit TapForAny branching to only relevant colors.
fn colors_needed_by_pieces(pieces: &[&CardDef]) -> Vec<Color> {
    let mut needed = Vec::new();
    for piece in pieces {
        for ability in &piece.activated_abilities {
            let c = &ability.cost;
            if c.white > 0 && !needed.contains(&Color::White) {
                needed.push(Color::White);
            }
            if c.blue > 0 && !needed.contains(&Color::Blue) {
                needed.push(Color::Blue);
            }
            if c.black > 0 && !needed.contains(&Color::Black) {
                needed.push(Color::Black);
            }
            if c.red > 0 && !needed.contains(&Color::Red) {
                needed.push(Color::Red);
            }
            if c.green > 0 && !needed.contains(&Color::Green) {
                needed.push(Color::Green);
            }
        }
    }
    needed
}

/// Count how many pieces have ManaFromNonlandBonus.
fn count_mana_bonus(pieces: &[&CardDef]) -> u32 {
    let mut count = 0u32;
    for piece in pieces {
        for sa in &piece.static_abilities {
            if matches!(sa, StaticAbility::ManaFromNonlandBonus) {
                count += 1;
            }
        }
    }
    count
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
            color_choice,
        } => {
            let piece = pieces[*piece_index];
            let ma = &piece.mana_abilities[*ability_index];
            let mana_desc = match ma {
                ManaAbility::TapForColor(c) => format!("{{{}}}", color_symbol(*c)),
                ManaAbility::TapForColorless => "{C}".to_string(),
                ManaAbility::TapForAny | ManaAbility::TapForChoice(_) => {
                    match color_choice {
                        Some(c) => format!("{{{}}}", color_symbol(*c)),
                        None => "{C}".to_string(),
                    }
                }
                ManaAbility::TapForColorlessAmount(n) => format!("{{{n}}}"),
            };
            format!("Tap {} for {}", piece.name, mana_desc)
        }
        ComboAction::ActivateAbility {
            piece_index,
            ability_index,
            untap_target,
            tapped_pieces_for_cost,
        } => {
            let piece = pieces[*piece_index];
            let ability = &piece.activated_abilities[*ability_index];
            let mut desc = format!("Activate {}: {}", piece.name, ability.description);
            if !tapped_pieces_for_cost.is_empty() {
                let tapped_names: Vec<&str> = tapped_pieces_for_cost
                    .iter()
                    .map(|&i| pieces[i].name.as_str())
                    .collect();
                desc.push_str(&format!(" (tapping {})", tapped_names.join(", ")));
            }
            if let Some(idx) = untap_target {
                desc.push_str(&format!(" (untap {})", pieces[*idx].name));
            }
            desc
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

        let mut i = size;
        loop {
            if i == 0 {
                return;
            }
            i -= 1;
            if indices[i] < items.len() - size + i {
                break;
            }
        }

        indices[i] += 1;
        for j in (i + 1)..size {
            indices[j] = indices[j - 1] + 1;
        }

        for (k, &idx) in indices.iter().enumerate() {
            buffer[k] = items[idx].clone();
        }
    }
}

/// Iterate over all combinations of `size` elements from `items` (by index).
/// Calls `callback` with each subset. When size is 0, calls callback once with empty slice.
fn for_each_combination_indices(items: &[usize], size: usize, mut callback: impl FnMut(&[usize])) {
    if size == 0 {
        callback(&[]);
        return;
    }
    if size > items.len() {
        return;
    }
    let mut indices: Vec<usize> = (0..size).collect();
    let mut buffer: Vec<usize> = indices.iter().map(|&i| items[i]).collect();

    loop {
        callback(&buffer);

        let mut i = size;
        loop {
            if i == 0 {
                return;
            }
            i -= 1;
            if indices[i] < items.len() - size + i {
                break;
            }
        }

        indices[i] += 1;
        for j in (i + 1)..size {
            indices[j] = indices[j - 1] + 1;
        }

        for (k, &idx) in indices.iter().enumerate() {
            buffer[k] = items[idx];
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
        assert!(
            combo.categories.contains(&ComboCategory::InfiniteMana),
            "Basalt + Kinnan should be categorized as InfiniteMana, got {:?}",
            combo.categories
        );
        assert_eq!(
            combo.categories.len(), 1,
            "Basalt + Kinnan should have exactly one category (InfiniteMana), got {:?}",
            combo.categories
        );
    }

    #[test]
    fn test_basalt_alone_is_not_infinite() {
        let db = sample::build_sample_db();
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

    // =====================================================================
    // Creature-based combo tests
    // =====================================================================

    #[test]
    fn test_discover_marrow_gnawer_thornbite_staff() {
        let db = sample::build_sample_db();
        let cards = vec![ids::MARROW_GNAWER, ids::THORNBITE_STAFF];
        let config = DiscoveryConfig::default();

        let combos = discover_combos(&db, &cards, &config);
        assert!(
            !combos.is_empty(),
            "Should discover Marrow-Gnawer + Thornbite Staff combo"
        );

        let combo = &combos[0];
        assert!(combo.pieces.contains(&ids::MARROW_GNAWER));
        assert!(combo.pieces.contains(&ids::THORNBITE_STAFF));
        assert!(
            combo.net_creatures_per_cycle > 0,
            "Should produce net creatures per cycle, got {}",
            combo.net_creatures_per_cycle
        );
        // Needs at least 2 creature tokens (Marrow-Gnawer counts as 1 creature,
        // plus 2 other rats to get exponential growth: sac 1, create 2+ tokens)
        assert!(
            combo.startup_creatures >= 2,
            "Should need at least 2 startup creatures, got {}",
            combo.startup_creatures
        );
        assert!(
            combo.categories.contains(&ComboCategory::InfiniteTokens),
            "Marrow-Gnawer + Thornbite should be categorized as InfiniteTokens, got {:?}",
            combo.categories
        );
        assert!(
            !combo.categories.contains(&ComboCategory::InfiniteDamage),
            "2-piece token combo should not include InfiniteDamage without a payoff piece"
        );
    }

    #[test]
    fn test_marrow_gnawer_alone_not_infinite() {
        let db = sample::build_sample_db();
        // Without Thornbite Staff, Marrow-Gnawer can't untap himself
        let cards = vec![ids::MARROW_GNAWER];
        let config = DiscoveryConfig {
            max_combo_pieces: 2,
            ..Default::default()
        };

        let combos = discover_combos(&db, &cards, &config);
        assert!(
            combos.is_empty(),
            "Marrow-Gnawer alone should not be an infinite combo"
        );
    }

    #[test]
    fn test_marrow_gnawer_thornbite_blood_artist() {
        let db = sample::build_sample_db();
        // With Blood Artist: each death during the cycle drains the opponent.
        // The 3-piece combo needs fewer startup creatures (1 vs 2) because
        // Blood Artist adds a piece creature, making CreaturesControlled
        // higher even at break-even token counts.
        let cards = vec![
            ids::MARROW_GNAWER,
            ids::THORNBITE_STAFF,
            ids::BLOOD_ARTIST,
        ];
        let config = DiscoveryConfig::default();

        let combos = discover_combos(&db, &cards, &config);
        assert!(
            !combos.is_empty(),
            "Should discover Marrow-Gnawer + Thornbite + Blood Artist"
        );

        // The 2-piece combo (Marrow+Staff) should still be found
        let two_piece = combos.iter().find(|c| c.pieces.len() == 2);
        assert!(
            two_piece.is_some(),
            "Should find the 2-piece combo"
        );

        // The 3-piece combo should also be found with damage output
        let three_piece = combos.iter().find(|c| {
            c.pieces.len() == 3 && c.pieces.contains(&ids::BLOOD_ARTIST)
        });
        assert!(
            three_piece.is_some(),
            "Should find the 3-piece combo with Blood Artist"
        );
        let three_piece = three_piece.unwrap();
        assert!(
            three_piece.damage_per_cycle > 0 || three_piece.life_per_cycle > 0,
            "3-piece combo should deal damage or gain life, got damage={} life={}",
            three_piece.damage_per_cycle,
            three_piece.life_per_cycle,
        );
        // Composite categories: tokens + damage + life gain
        assert!(
            three_piece.categories.contains(&ComboCategory::InfiniteTokens),
            "Blood Artist combo should include InfiniteTokens, got {:?}",
            three_piece.categories
        );
        assert!(
            three_piece.categories.contains(&ComboCategory::InfiniteDamage)
                || three_piece.categories.contains(&ComboCategory::InfiniteLifeGain),
            "Blood Artist combo should include InfiniteDamage or InfiniteLifeGain, got {:?}",
            three_piece.categories
        );
        // The 3-piece combo needs fewer startup creatures than the 2-piece
        assert!(
            three_piece.startup_creatures < two_piece.unwrap().startup_creatures,
            "3-piece combo should need fewer startup creatures ({}) than 2-piece ({})",
            three_piece.startup_creatures,
            two_piece.unwrap().startup_creatures,
        );
    }

    // =====================================================================
    // Utility tests
    // =====================================================================

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
        assert_eq!(results.len(), 1);
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

        let c = ManaPool {
            colorless: 5,
            ..ManaPool::empty()
        };
        assert!(mana_pool_ge(&a, &c));
    }

    #[test]
    fn test_no_combos_among_vanilla_creatures() {
        let db = sample::build_sample_db();
        let cards = vec![ids::GRIZZLY_BEARS, ids::GREY_OGRE, ids::SERRA_ANGEL];
        let config = DiscoveryConfig::default();

        let combos = discover_combos(&db, &cards, &config);
        assert!(
            combos.is_empty(),
            "Vanilla creatures should not produce combos"
        );
    }

    #[test]
    fn test_supersets_not_pruned() {
        // Supersets are no longer pruned because a 3-piece combo can be
        // meaningfully different from a 2-piece subset (e.g., adding a
        // drain-on-death effect). Both the 2-piece and 3-piece combos
        // should be discovered.
        let db = sample::build_sample_db();
        let cards = vec![
            ids::BASALT_MONOLITH,
            ids::KINNAN_BONDER_PRODIGY,
            ids::THRASIOS_TRITON_HERO,
        ];
        let config = DiscoveryConfig::default();

        let combos = discover_combos(&db, &cards, &config);

        let two_piece = combos.iter().find(|c| c.pieces.len() == 2);
        assert!(
            two_piece.is_some(),
            "Should still find the 2-piece combo"
        );
    }

    #[test]
    fn test_marrow_gnawer_thornbite_ayara_etb_triggers() {
        let db = sample::build_sample_db();
        // Ayara triggers on ACreatureEnters — each rat token entering
        // drains the opponent for 1. This tests that ETB triggers fire
        // during combo discovery when tokens are created.
        let cards = vec![
            ids::MARROW_GNAWER,
            ids::THORNBITE_STAFF,
            ids::AYARA_FIRST_OF_LOCTHWAIN,
        ];
        let config = DiscoveryConfig::default();

        let combos = discover_combos(&db, &cards, &config);

        let three_piece = combos.iter().find(|c| {
            c.pieces.len() == 3 && c.pieces.contains(&ids::AYARA_FIRST_OF_LOCTHWAIN)
        });
        assert!(
            three_piece.is_some(),
            "Should find the 3-piece combo with Ayara"
        );
        let three_piece = three_piece.unwrap();
        assert!(
            three_piece.damage_per_cycle > 0 || three_piece.life_per_cycle > 0,
            "Ayara ETB should produce damage/life per cycle, got damage={} life={}",
            three_piece.damage_per_cycle,
            three_piece.life_per_cycle,
        );
        // Composite categories: tokens + damage + life gain
        assert!(
            three_piece.categories.contains(&ComboCategory::InfiniteTokens),
            "Ayara combo should include InfiniteTokens, got {:?}",
            three_piece.categories
        );
        assert!(
            three_piece.categories.contains(&ComboCategory::InfiniteDamage)
                || three_piece.categories.contains(&ComboCategory::InfiniteLifeGain),
            "Ayara combo should include InfiniteDamage or InfiniteLifeGain, got {:?}",
            three_piece.categories
        );
    }

    #[test]
    fn test_equipment_attachment_configs() {
        // 1 equipment, 2 creatures → 2 configs
        let configs = generate_attachment_configs(&[0], &[1, 2]);
        assert_eq!(configs.len(), 2);
        assert!(configs.contains(&vec![(0, 1)]));
        assert!(configs.contains(&vec![(0, 2)]));
    }
}
