//! MCTS (Monte Carlo Tree Search) for goldfish/solitaire optimization.
//!
//! Unlike MCCFR which finds Nash equilibria for adversarial games, MCTS is
//! appropriate for single-agent optimization where the goal is to find the
//! best sequence of actions to maximize a reward (here: minimize kill turn).
//!
//! # Algorithm
//!
//! For each decision point, we build a search tree rooted at the current
//! game state and repeatedly:
//!
//! 1. **Select** — Walk down the tree using UCB1 to balance exploration
//!    and exploitation.
//! 2. **Expand** — When we reach a leaf node, expand it by adding children
//!    for each legal action.
//! 3. **Simulate** — Play out the rest of the game from the expanded node
//!    using a rollout policy (GreedyStrategy by default).
//! 4. **Backpropagate** — Update visit counts and reward sums back up the
//!    path from the expanded node to the root.
//!
//! Since this is solitaire (no adversary), all nodes are maximization nodes.
//! The goldfish opponent's actions are deterministic (always pass), so we
//! skip tree branching for their decisions.
//!
//! # Reward Function
//!
//! - Win on turn T: `reward = (MAX_TURN + 1 - T) / MAX_TURN`
//!   Faster kills get higher rewards (range: ~0.05 to 1.0).
//! - Draw/loss: partial credit for life reduction, scaled to `[0, 0.04]`
//!   so any actual kill always beats any non-kill.

use rand::seq::SliceRandom;
use rayon::prelude::*;

use crate::action::{legal_actions, Action};
use crate::game::{GameFormat, GameState, Phase, PlayerIndex};
use crate::rules;
use crate::simulation::format_action_name;
use crate::strategy::{GoldfishStrategy, GreedyStrategy, Strategy};

/// Maximum turns for goldfish MCTS games (matches simulation module).
const GOLDFISH_MAX_TURNS: u32 = 20;

/// Maximum actions per game to prevent infinite loops.
const GOLDFISH_MAX_ACTIONS: u32 = 10_000;

// ---------------------------------------------------------------------------
// Configuration
// ---------------------------------------------------------------------------

/// Configuration for MCTS search.
#[derive(Debug, Clone)]
pub struct MctsConfig {
    /// Number of MCTS iterations (tree walks) per decision point.
    /// More iterations = better play but slower decisions.
    pub iterations_per_move: u32,

    /// UCB1 exploration constant. Higher values explore more, lower values
    /// exploit known-good actions. sqrt(2) ≈ 1.414 is theoretically optimal
    /// for rewards in [0, 1]; 0.5–1.0 often works well in practice.
    pub exploration_constant: f64,

    /// Maximum depth of the search tree (in player-0 decisions, not total
    /// actions). 0 = unlimited. Deeper trees find longer-horizon plays
    /// but use more memory and time.
    pub max_tree_depth: u32,

    /// Maximum number of actions during rollout before declaring a draw.
    /// Prevents runaway rollouts in degenerate game states.
    pub max_rollout_actions: u32,

    /// Number of threads for parallel MCTS (root parallelization).
    /// Each thread builds an independent search tree with
    /// `iterations_per_move / num_threads` iterations, then results are
    /// merged by summing per-action visit counts and rewards.
    /// 0 or 1 = single-threaded (default). Values > 1 enable parallelism.
    pub num_threads: u32,
}

impl Default for MctsConfig {
    fn default() -> Self {
        MctsConfig {
            iterations_per_move: 500,
            exploration_constant: 1.0,
            max_tree_depth: 0,
            max_rollout_actions: 5_000,
            num_threads: 1,
        }
    }
}

// ---------------------------------------------------------------------------
// Tree structure
// ---------------------------------------------------------------------------

/// A node in the MCTS search tree.
///
/// Each node represents a game state after a sequence of actions from the
/// root. In goldfish mode, only the pilot (player 0) has meaningful
/// decisions; goldfish actions are applied deterministically without
/// creating tree branches.
#[derive(Debug)]
pub(crate) struct MctsNode {
    /// Number of times this node has been visited during search.
    visits: u32,

    /// Sum of rewards from all rollouts through this node.
    /// Reward is in [0, 1] where higher = faster kill.
    total_reward: f64,

    /// Children indexed by action. Populated on first visit (expansion).
    /// `None` means this node hasn't been expanded yet.
    children: Option<Vec<MctsChild>>,
}

/// A child edge in the MCTS tree (action + resulting node).
#[derive(Debug)]
struct MctsChild {
    /// The action that transitions from the parent to this child.
    action: Action,
    /// The child node.
    node: MctsNode,
}

impl MctsNode {
    fn new() -> Self {
        MctsNode {
            visits: 0,
            total_reward: 0.0,
            children: None,
        }
    }

    /// Average reward (Q-value) for this node.
    fn avg_reward(&self) -> f64 {
        if self.visits == 0 {
            0.0
        } else {
            self.total_reward / self.visits as f64
        }
    }

    /// Whether this node has been expanded (children generated).
    fn is_expanded(&self) -> bool {
        self.children.is_some()
    }
}

// ---------------------------------------------------------------------------
// UCB1 selection
// ---------------------------------------------------------------------------

/// Select the child index with the highest UCB1 score.
///
/// UCB1 = Q_i + C * sqrt(ln(N_parent) / N_i)
///
/// where Q_i is the average reward of child i, N_parent is the parent's
/// visit count, N_i is the child's visit count, and C is the exploration
/// constant. Unvisited children get infinite UCB1 (selected first).
///
/// Ties are broken randomly to improve search diversity (e.g., when
/// multiple children are unvisited, they're explored in random order
/// rather than always left-to-right).
fn ucb1_select(children: &[MctsChild], parent_visits: u32, c: f64) -> usize {
    let ln_parent = (parent_visits as f64).ln();

    // Compute UCB1 for each child
    let ucb_scores: Vec<f64> = children
        .iter()
        .map(|child| {
            if child.node.visits == 0 {
                f64::INFINITY
            } else {
                child.node.avg_reward() + c * (ln_parent / child.node.visits as f64).sqrt()
            }
        })
        .collect();

    let best_ucb = ucb_scores
        .iter()
        .cloned()
        .fold(f64::NEG_INFINITY, f64::max);

    // Collect all indices tied at the best score
    let tied: Vec<usize> = ucb_scores
        .iter()
        .enumerate()
        .filter(|(_, &score)| {
            // For infinity (unvisited), check with is_infinite
            if best_ucb.is_infinite() {
                score.is_infinite()
            } else {
                (score - best_ucb).abs() < 1e-12
            }
        })
        .map(|(i, _)| i)
        .collect();

    // Random tie-breaking among best
    let mut rng = rand::thread_rng();
    *tied.choose(&mut rng).unwrap_or(&0)
}

// ---------------------------------------------------------------------------
// Reward function
// ---------------------------------------------------------------------------

/// Compute reward for a completed goldfish game.
///
/// Returns a value in [0, 1] where:
/// - Win on turn T: `(MAX_TURN + 1 - T) / MAX_TURN` — faster kills get
///   higher reward. Turn 1 kill = 1.0, turn 20 kill = 0.05.
/// - No kill: partial credit for life reduction, scaled to `[0, 0.04]` so
///   any actual kill (min 0.05) always outranks any non-kill. This gives
///   MCTS gradient signal even when rollouts can't close the game.
fn goldfish_reward(
    winner: Option<PlayerIndex>,
    turn: u32,
    opponent_life: i32,
    starting_life: i32,
) -> f64 {
    match winner {
        Some(0) => {
            let t = turn.min(GOLDFISH_MAX_TURNS);
            (GOLDFISH_MAX_TURNS + 1 - t) as f64 / GOLDFISH_MAX_TURNS as f64
        }
        _ => {
            // Reward shaping: partial credit for damage dealt.
            // Scale to [0, 0.04] — strictly below worst win (T20 = 0.05).
            let damage = (starting_life - opponent_life).max(0) as f64;
            let fraction = (damage / starting_life as f64).min(1.0);
            fraction * 0.04
        }
    }
}

/// Starting life total for a game format.
fn format_starting_life(format: GameFormat) -> i32 {
    match format {
        GameFormat::Commander => 40,
        GameFormat::Standard => 20,
    }
}

// ---------------------------------------------------------------------------
// Core MCTS algorithm
// ---------------------------------------------------------------------------

/// Run MCTS from the given game state and return the best action.
///
/// This is the main entry point for a single MCTS decision. It builds
/// (or reuses) a search tree, runs `config.iterations_per_move` iterations,
/// and returns the action with the highest visit count (most robust child).
///
/// When `config.num_threads > 1`, uses root parallelization: each thread
/// builds an independent tree with a share of the iterations, then results
/// are merged by summing per-action visit counts and rewards.
pub(crate) fn mcts_search(
    state: &GameState,
    config: &MctsConfig,
    root: &mut MctsNode,
) -> Option<Action> {
    let actions = legal_actions(state);
    if actions.is_empty() {
        return Some(Action::PassPriority);
    }
    if actions.len() == 1 {
        return Some(actions[0].clone());
    }

    let num_threads = config.num_threads.max(1);

    if num_threads <= 1 {
        // Single-threaded path (original behavior)
        mcts_search_single(state, config, root, &actions)
    } else {
        // Parallel root parallelization
        mcts_search_parallel(state, config, root, &actions, num_threads)
    }
}

/// Single-threaded MCTS search. Runs all iterations on one tree.
fn mcts_search_single(
    state: &GameState,
    config: &MctsConfig,
    root: &mut MctsNode,
    actions: &[Action],
) -> Option<Action> {
    // Expand root if needed
    if !root.is_expanded() {
        expand_node(root, actions);
    }

    let greedy = GreedyStrategy;
    let goldfish = GoldfishStrategy;

    for _ in 0..config.iterations_per_move {
        // Clone the state for this iteration (each iteration modifies state)
        let mut sim_state = state.clone();
        let reward = tree_walk(&mut sim_state, root, config, &greedy, &goldfish, 0);
        root.visits += 1;
        root.total_reward += reward;
    }

    // Select the action with the most visits (most robust child)
    best_action_by_visits(root)
}

/// Parallel MCTS search using root parallelization.
///
/// Each thread builds an independent search tree with a share of the total
/// iterations. After all threads complete, per-action visit counts and
/// rewards are summed to select the most robust action.
///
/// This is the simplest and most effective parallelization strategy for MCTS:
/// - No locks on tree nodes (each tree is thread-local)
/// - Linear speedup proportional to thread count
/// - Slightly more total iterations due to rounding, but never fewer
///
/// **Root overwrite:** The passed-in `root` node is overwritten with merged
/// statistics from all threads. This is fine because both callers
/// (`MctsStrategy::choose_action` and `run_mcts_goldfish_game`) create a
/// fresh `MctsNode::new()` per decision — tree reuse across decisions is
/// not supported. The root is populated solely so callers can read decision
/// stats (visit counts, avg reward) for logging.
///
/// **Nested parallelism:** This uses Rayon's `into_par_iter`, which shares
/// the global thread pool with the outer game-level parallelism in
/// `simulate_mcts_goldfish`. When many games run concurrently, inner MCTS
/// parallelism won't get additional threads (the pool is already saturated).
/// `num_threads > 1` is most useful when running a single game or few games.
fn mcts_search_parallel(
    state: &GameState,
    config: &MctsConfig,
    root: &mut MctsNode,
    actions: &[Action],
    num_threads: u32,
) -> Option<Action> {
    let iters_per_thread = config.iterations_per_move / num_threads;
    let remainder = config.iterations_per_move % num_threads;

    // Each thread runs its own tree search and returns per-action (visits, total_reward).
    let thread_results: Vec<Vec<(u32, f64)>> = (0..num_threads)
        .into_par_iter()
        .map(|thread_idx| {
            // First `remainder` threads get one extra iteration
            let my_iters = iters_per_thread + if thread_idx < remainder { 1 } else { 0 };
            if my_iters == 0 {
                return vec![(0, 0.0); actions.len()];
            }

            let mut thread_root = MctsNode::new();
            expand_node(&mut thread_root, actions);

            // Single-threaded config for each thread's tree (avoid nested parallelism)
            let thread_config = MctsConfig {
                num_threads: 1,
                ..*config
            };

            let greedy = GreedyStrategy;
            let goldfish = GoldfishStrategy;

            for _ in 0..my_iters {
                let mut sim_state = state.clone();
                let reward = tree_walk(
                    &mut sim_state,
                    &mut thread_root,
                    &thread_config,
                    &greedy,
                    &goldfish,
                    0,
                );
                thread_root.visits += 1;
                thread_root.total_reward += reward;
            }

            // Extract per-action stats
            thread_root
                .children
                .as_ref()
                .map(|children| {
                    children
                        .iter()
                        .map(|c| (c.node.visits, c.node.total_reward))
                        .collect()
                })
                .unwrap_or_else(|| vec![(0, 0.0); actions.len()])
        })
        .collect();

    // Merge results: sum visits and rewards per action across all threads
    let num_actions = actions.len();
    let mut merged_visits = vec![0u32; num_actions];
    let mut merged_rewards = vec![0.0f64; num_actions];

    for thread_result in &thread_results {
        for (i, &(visits, reward)) in thread_result.iter().enumerate() {
            if i < num_actions {
                merged_visits[i] += visits;
                merged_rewards[i] += reward;
            }
        }
    }

    // Update the root node with merged statistics
    if !root.is_expanded() {
        expand_node(root, actions);
    }
    let total_visits: u32 = merged_visits.iter().sum();
    let total_reward: f64 = merged_rewards.iter().sum();
    root.visits = total_visits;
    root.total_reward = total_reward;

    if let Some(children) = root.children.as_mut() {
        for (i, child) in children.iter_mut().enumerate() {
            if i < num_actions {
                child.node.visits = merged_visits[i];
                child.node.total_reward = merged_rewards[i];
            }
        }
    }

    // Select the action with the most total visits across all trees
    best_action_by_visits(root)
}

/// Perform one MCTS iteration: select → expand → simulate → backpropagate.
///
/// Returns the reward obtained from this iteration. The caller is responsible
/// for updating the root node's statistics.
///
/// Forced passes and goldfish turns are handled iteratively (loop) to avoid
/// unbounded recursion that could overflow the stack. Only player-0 decision
/// points use recursion (bounded by tree depth).
fn tree_walk(
    state: &mut GameState,
    node: &mut MctsNode,
    config: &MctsConfig,
    rollout_strategy: &dyn Strategy,
    goldfish_strategy: &GoldfishStrategy,
    depth: u32,
) -> f64 {
    // Advance past forced passes and goldfish turns iteratively.
    // This loop replaces what was previously tail-recursion through
    // non-decision states, preventing O(phases × turns) stack depth.
    loop {
        // Terminal check
        if state.game_over || state.turn_number > GOLDFISH_MAX_TURNS {
            let sl = format_starting_life(state.format);
            return goldfish_reward(state.winner, state.turn_number, state.players[1].life, sl);
        }

        // Depth limit — switch to rollout
        if config.max_tree_depth > 0 && depth >= config.max_tree_depth {
            return rollout(state, rollout_strategy, goldfish_strategy, config);
        }

        let player = state.priority_player;
        let actions = legal_actions(state);

        // No actions or only pass — advance without branching
        if actions.is_empty()
            || (actions.len() == 1 && actions[0] == Action::PassPriority)
        {
            rules::apply_action(state, &Action::PassPriority);
            continue;
        }

        // Goldfish (player 1) — deterministic, no branching
        if player != 0 {
            let action = goldfish_strategy.choose_action(state, player);
            rules::apply_action(state, &action);
            continue;
        }

        // Mulligan phase — handle with rollout strategy, no tree branching.
        // MulliganMulligan introduces stochasticity (random shuffle/draw) that
        // breaks MCTS tree reuse: the same tree node would be visited with
        // different hands across iterations, causing action mismatches that
        // corrupt the mulligan state machine. The GreedyStrategy heuristic
        // handles mulligans well, so we skip tree search for this phase.
        if state.phase == Phase::Mulligan {
            let action = rollout_strategy.choose_action(state, player);
            rules::apply_action(state, &action);
            continue;
        }

        // --- Player 0 decision node — break out of loop to handle below ---
        break;
    }

    // We only reach here at a player-0 decision node.
    let actions = legal_actions(state);

    // Expand if this is a leaf
    if !node.is_expanded() {
        expand_node(node, &actions);
        // First visit to a new node: rollout from here.
        // Don't update node stats here — the caller's backpropagation handles it.
        return rollout(state, rollout_strategy, goldfish_strategy, config);
    }

    let children = node.children.as_mut().unwrap();

    // If our action set changed (e.g., different game path led here), re-expand.
    // This shouldn't happen in practice for goldfish since the tree is built
    // deterministically, but handle it defensively.
    if children.is_empty() {
        return rollout(state, rollout_strategy, goldfish_strategy, config);
    }

    // Select child using UCB1
    let child_idx = ucb1_select(children, node.visits.max(1), config.exploration_constant);

    // Apply the selected action
    let action = children[child_idx].action.clone();
    rules::apply_action(state, &action);

    // Recurse (bounded by tree depth — only player-0 decisions increment depth)
    let reward = tree_walk(
        state,
        &mut children[child_idx].node,
        config,
        rollout_strategy,
        goldfish_strategy,
        depth + 1,
    );

    // Backpropagate
    children[child_idx].node.visits += 1;
    children[child_idx].node.total_reward += reward;

    reward
}

/// Expand a node by creating child entries for each legal action.
fn expand_node(node: &mut MctsNode, actions: &[Action]) {
    let children: Vec<MctsChild> = actions
        .iter()
        .map(|a| MctsChild {
            action: a.clone(),
            node: MctsNode::new(),
        })
        .collect();
    node.children = Some(children);
}

/// Run a rollout (simulation) from the current state to completion using
/// the rollout policy, and return the reward.
fn rollout(
    state: &mut GameState,
    rollout_strategy: &dyn Strategy,
    goldfish_strategy: &GoldfishStrategy,
    config: &MctsConfig,
) -> f64 {
    let mut actions_taken: u32 = 0;

    while !state.game_over
        && state.turn_number <= GOLDFISH_MAX_TURNS
        && actions_taken < config.max_rollout_actions
    {
        let player = state.priority_player;
        let actions = legal_actions(state);

        if actions.is_empty()
            || (actions.len() == 1 && actions[0] == Action::PassPriority)
        {
            rules::apply_action(state, &Action::PassPriority);
            actions_taken += 1;
            continue;
        }

        let strategy: &dyn Strategy = if player == 0 {
            rollout_strategy
        } else {
            goldfish_strategy
        };
        let action = strategy.choose_action(state, player);
        rules::apply_action(state, &action);
        actions_taken += 1;

        if actions_taken % 10 == 0 {
            rules::check_state_based_actions(state);
        }
    }

    let sl = format_starting_life(state.format);
    goldfish_reward(state.winner, state.turn_number, state.players[1].life, sl)
}

/// Select the action with the highest visit count (most robust child selection).
///
/// In MCTS, the most-visited child is preferred over the highest-average-reward
/// child because visit count is more robust to noise from rollouts.
fn best_action_by_visits(root: &MctsNode) -> Option<Action> {
    let children = root.children.as_ref()?;
    children
        .iter()
        .max_by_key(|c| c.node.visits)
        .map(|c| c.action.clone())
}

// ---------------------------------------------------------------------------
// MctsStrategy — implements the Strategy trait
// ---------------------------------------------------------------------------

/// MCTS-based strategy for goldfish solitaire optimization.
///
/// At each decision point, runs MCTS from the current game state to find
/// the best action. This is an "online" planner — it builds a fresh search
/// tree at each decision point.
///
/// For goldfish, only player 0's decisions matter. When used as player 1's
/// strategy (shouldn't happen), falls back to GoldfishStrategy.
pub struct MctsStrategy {
    config: MctsConfig,
}

impl MctsStrategy {
    pub fn new(config: MctsConfig) -> Self {
        MctsStrategy { config }
    }
}

impl Strategy for MctsStrategy {
    fn choose_action(&self, state: &GameState, player: PlayerIndex) -> Action {
        // MCTS only makes sense for the pilot (player 0)
        if player != 0 {
            return GoldfishStrategy.choose_action(state, player);
        }

        // Mulligan phase — delegate to GreedyStrategy. MCTS tree search is
        // unsound for mulligans because MulliganMulligan introduces stochastic
        // shuffle/draw that breaks tree node reuse across iterations.
        if state.phase == Phase::Mulligan {
            return GreedyStrategy.choose_action(state, player);
        }

        let actions = legal_actions(state);
        if actions.is_empty() {
            return Action::PassPriority;
        }
        if actions.len() == 1 {
            return actions[0].clone();
        }

        let mut root = MctsNode::new();
        mcts_search(state, &self.config, &mut root).unwrap_or(Action::PassPriority)
    }

    fn name(&self) -> &str {
        "MCTS"
    }
}

// ---------------------------------------------------------------------------
// Full-game MCTS search: find optimal play for a fixed shuffle
// ---------------------------------------------------------------------------

/// Result of a single MCTS goldfish game.
#[derive(Debug, Clone)]
pub struct MctsGameResult {
    /// Whether player 0 won.
    pub won: bool,
    /// Turn the game ended.
    pub kill_turn: u32,
    /// Total actions taken.
    pub actions_taken: u32,
    /// Final life totals.
    pub final_life: [i32; 2],
    /// Per-decision statistics: (turn, phase, num_actions_considered, iterations_used, best_action_visits, best_action_avg_reward)
    pub decision_stats: Vec<DecisionStat>,
}

/// Statistics for a single MCTS decision point.
#[derive(Debug, Clone)]
pub struct DecisionStat {
    pub turn: u32,
    pub phase: Phase,
    pub num_legal_actions: usize,
    pub best_action_visits: u32,
    pub best_action_avg_reward: f64,
    pub action_description: String,
}

/// Run a single goldfish game using MCTS for player 0's decisions.
///
/// For a fixed shuffle (determined by the caller), this finds near-optimal
/// play by running MCTS at each decision point. Returns detailed statistics
/// about each decision.
pub fn run_mcts_goldfish_game(
    state: &mut GameState,
    config: &MctsConfig,
    verbose: bool,
) -> MctsGameResult {
    let goldfish = GoldfishStrategy;
    let mut actions_taken: u32 = 0;
    let mut decision_stats = Vec::new();

    while !state.game_over
        && state.turn_number <= GOLDFISH_MAX_TURNS
        && actions_taken < GOLDFISH_MAX_ACTIONS
    {
        let player = state.priority_player;
        let actions = legal_actions(state);

        if actions.is_empty()
            || (actions.len() == 1 && actions[0] == Action::PassPriority)
        {
            rules::apply_action(state, &Action::PassPriority);
            actions_taken += 1;
            continue;
        }

        if player != 0 {
            // Goldfish — deterministic, no search needed
            let action = goldfish.choose_action(state, player);
            rules::apply_action(state, &action);
            actions_taken += 1;
            continue;
        }

        // Mulligan phase — use GreedyStrategy heuristic directly.
        // MCTS tree search over mulligan decisions is unsound because
        // MulliganMulligan involves stochastic shuffle/draw, causing
        // tree nodes to be visited with different hands across iterations.
        if state.phase == Phase::Mulligan {
            let greedy = GreedyStrategy;
            let action = greedy.choose_action(state, player);
            if verbose {
                eprintln!(
                    "T{} {:?} P0: {} (greedy mulligan)",
                    state.turn_number,
                    state.phase,
                    format_action(&action, state),
                );
            }
            rules::apply_action(state, &action);
            actions_taken += 1;
            continue;
        }

        // Player 0 decision — run MCTS
        let action = if actions.len() == 1 {
            actions[0].clone()
        } else {
            let mut root = MctsNode::new();
            let best = mcts_search(state, config, &mut root)
                .unwrap_or(Action::PassPriority);

            // Collect decision statistics
            if let Some(children) = &root.children {
                let best_child = children.iter().max_by_key(|c| c.node.visits);
                if let Some(bc) = best_child {
                    let stat = DecisionStat {
                        turn: state.turn_number,
                        phase: state.phase,
                        num_legal_actions: children.len(),
                        best_action_visits: bc.node.visits,
                        best_action_avg_reward: bc.node.avg_reward(),
                        action_description: format_action(&best, state),
                    };
                    decision_stats.push(stat);
                }
            }

            if verbose {
                eprintln!(
                    "T{} {:?} P0: {} (of {} actions, {}/{} iters)",
                    state.turn_number,
                    state.phase,
                    format_action(&best, state),
                    actions.len(),
                    root.children
                        .as_ref()
                        .and_then(|c| c.iter().max_by_key(|c| c.node.visits))
                        .map(|c| c.node.visits)
                        .unwrap_or(0),
                    config.iterations_per_move,
                );
            }

            best
        };

        rules::apply_action(state, &action);
        actions_taken += 1;

        if actions_taken % 10 == 0 {
            rules::check_state_based_actions(state);
        }
    }

    MctsGameResult {
        won: state.winner == Some(0),
        kill_turn: state.turn_number,
        actions_taken,
        final_life: [state.players[0].life, state.players[1].life],
        decision_stats,
    }
}

/// Format an action for human-readable display.
/// Delegates to the shared `format_action_name` in the simulation module.
fn format_action(action: &Action, state: &GameState) -> String {
    format_action_name(state, action)
}

// ---------------------------------------------------------------------------
// Aggregate MCTS goldfish statistics
// ---------------------------------------------------------------------------

/// Aggregate results from many MCTS goldfish games.
#[derive(Debug, Clone)]
pub struct MctsGoldfishResults {
    pub total_games: u64,
    pub wins: u64,
    pub losses: u64,
    pub draws: u64,
    pub avg_kill_turn: f64,
    pub fastest_kill: u32,
    pub slowest_kill: u32,
    pub avg_actions: f64,
    pub avg_decisions_per_game: f64,
    pub avg_best_reward: f64,
    /// Kill-turn distribution: index = turn number, value = number of wins.
    pub kill_turn_distribution: Vec<u64>,
}

impl MctsGoldfishResults {
    pub fn win_rate(&self) -> f64 {
        if self.total_games == 0 {
            return 0.0;
        }
        self.wins as f64 / self.total_games as f64
    }

    pub fn display(&self) {
        println!("=== MCTS Goldfish Results ===");
        println!("Total games: {}", self.total_games);
        println!("Wins: {} ({:.1}%)", self.wins, self.win_rate() * 100.0);
        println!("Draws (timeout): {}", self.draws);
        if self.wins > 0 {
            println!("Avg kill turn: {:.2}", self.avg_kill_turn);
            println!("Fastest kill: T{}", self.fastest_kill);
            println!("Slowest kill: T{}", self.slowest_kill);
            println!("Avg actions/game: {:.1}", self.avg_actions);
            println!("Avg decisions/game: {:.1}", self.avg_decisions_per_game);
            println!("Avg best-action reward: {:.4}", self.avg_best_reward);
            println!("Kill turn distribution:");
            for (turn, &count) in self.kill_turn_distribution.iter().enumerate() {
                if count > 0 {
                    let pct = count as f64 / self.wins as f64 * 100.0;
                    println!("  T{}: {} ({:.1}%)", turn, count, pct);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_goldfish_reward() {
        // Win on turn 1 → highest reward (opponent life irrelevant for wins)
        assert!((goldfish_reward(Some(0), 1, 0, 20) - 1.0).abs() < 1e-10);
        // Win on turn 20 → lowest positive reward
        assert!((goldfish_reward(Some(0), 20, 0, 20) - 1.0 / 20.0).abs() < 1e-10);
        // Win on turn 5
        assert!((goldfish_reward(Some(0), 5, 0, 20) - 16.0 / 20.0).abs() < 1e-10);
        // Loss with no damage → 0.0
        assert!((goldfish_reward(Some(1), 5, 20, 20)).abs() < 1e-10);
        // Draw with no damage → 0.0
        assert!((goldfish_reward(None, 20, 20, 20)).abs() < 1e-10);
    }

    #[test]
    fn test_goldfish_reward_shaping() {
        // No kill, no damage → 0.0
        assert!((goldfish_reward(None, 20, 40, 40)).abs() < 1e-10);
        // No kill, half damage (40 → 20) → 0.02
        assert!((goldfish_reward(None, 20, 20, 40) - 0.02).abs() < 1e-10);
        // No kill, most damage (40 → 1) → 39/40 * 0.04 = 0.039
        assert!((goldfish_reward(None, 20, 1, 40) - 39.0 / 40.0 * 0.04).abs() < 1e-10);
        // Shaping reward is always less than worst win (T20 = 0.05)
        let worst_win = goldfish_reward(Some(0), 20, 0, 40);
        let best_shaping = goldfish_reward(None, 20, 0, 40);
        assert!(best_shaping < worst_win);
        // Standard format: no kill, half damage (20 → 10) → 0.02
        assert!((goldfish_reward(None, 20, 10, 20) - 0.02).abs() < 1e-10);
    }

    #[test]
    fn test_ucb1_select_unvisited_first() {
        // Unvisited children should be selected before visited ones
        let children = vec![
            MctsChild {
                action: Action::PassPriority,
                node: MctsNode {
                    visits: 10,
                    total_reward: 5.0,
                    children: None,
                },
            },
            MctsChild {
                action: Action::PassPriority,
                node: MctsNode::new(), // visits = 0
            },
        ];
        // Unvisited child (index 1) should be selected
        assert_eq!(ucb1_select(&children, 10, 1.0), 1);
    }

    #[test]
    fn test_ucb1_select_exploitation() {
        // With very low exploration constant, prefer high-reward child
        let children = vec![
            MctsChild {
                action: Action::PassPriority,
                node: MctsNode {
                    visits: 100,
                    total_reward: 90.0, // avg = 0.9
                    children: None,
                },
            },
            MctsChild {
                action: Action::PassPriority,
                node: MctsNode {
                    visits: 100,
                    total_reward: 10.0, // avg = 0.1
                    children: None,
                },
            },
        ];
        // With c=0 (pure exploitation), prefer the higher-reward child
        assert_eq!(ucb1_select(&children, 200, 0.0), 0);
    }

    #[test]
    fn test_mcts_node_avg_reward() {
        let node = MctsNode {
            visits: 10,
            total_reward: 7.5,
            children: None,
        };
        assert!((node.avg_reward() - 0.75).abs() < 1e-10);

        let empty = MctsNode::new();
        assert!((empty.avg_reward()).abs() < 1e-10);
    }

    #[test]
    fn test_expand_node() {
        let mut node = MctsNode::new();
        assert!(!node.is_expanded());

        let actions = vec![Action::PassPriority, Action::PlayLand { object_id: 42 }];
        expand_node(&mut node, &actions);

        assert!(node.is_expanded());
        let children = node.children.as_ref().unwrap();
        assert_eq!(children.len(), 2);
        assert_eq!(children[0].action, Action::PassPriority);
    }

    #[test]
    fn test_mcts_config_num_threads_default() {
        let config = MctsConfig::default();
        assert_eq!(config.num_threads, 1);
    }

    #[test]
    fn test_parallel_merge_sums_visits() {
        // Verify that mcts_search_parallel correctly merges per-action stats.
        // We test the merge logic directly by constructing thread results.
        let actions = vec![
            Action::PassPriority,
            Action::PlayLand { object_id: 1 },
            Action::PlayLand { object_id: 2 },
        ];

        let mut root = MctsNode::new();
        expand_node(&mut root, &actions);

        // Simulate two threads' results
        let thread_results: Vec<Vec<(u32, f64)>> = vec![
            vec![(10, 5.0), (20, 8.0), (5, 2.0)],
            vec![(15, 7.0), (10, 4.0), (8, 3.0)],
        ];

        // Apply merge logic (same as mcts_search_parallel)
        let num_actions = actions.len();
        let mut merged_visits = vec![0u32; num_actions];
        let mut merged_rewards = vec![0.0f64; num_actions];

        for thread_result in &thread_results {
            for (i, &(visits, reward)) in thread_result.iter().enumerate() {
                if i < num_actions {
                    merged_visits[i] += visits;
                    merged_rewards[i] += reward;
                }
            }
        }

        assert_eq!(merged_visits, vec![25, 30, 13]);
        assert!((merged_rewards[0] - 12.0).abs() < 1e-10);
        assert!((merged_rewards[1] - 12.0).abs() < 1e-10);
        assert!((merged_rewards[2] - 5.0).abs() < 1e-10);

        // Update root and verify best action is PlayLand{1} (most visits = 30)
        let total_visits: u32 = merged_visits.iter().sum();
        let total_reward: f64 = merged_rewards.iter().sum();
        root.visits = total_visits;
        root.total_reward = total_reward;

        if let Some(children) = root.children.as_mut() {
            for (i, child) in children.iter_mut().enumerate() {
                child.node.visits = merged_visits[i];
                child.node.total_reward = merged_rewards[i];
            }
        }

        let best = best_action_by_visits(&root);
        assert_eq!(best, Some(Action::PlayLand { object_id: 1 }));
    }

    #[test]
    fn test_iteration_distribution_across_threads() {
        // Verify iterations are split correctly with remainder handling.
        let total_iters: u32 = 103;
        let num_threads: u32 = 4;
        let base = total_iters / num_threads; // 25
        let remainder = total_iters % num_threads; // 3

        let mut assigned: Vec<u32> = Vec::new();
        for t in 0..num_threads {
            assigned.push(base + if t < remainder { 1 } else { 0 });
        }

        // First 3 threads get 26, last gets 25
        assert_eq!(assigned, vec![26, 26, 26, 25]);
        assert_eq!(assigned.iter().sum::<u32>(), total_iters);
    }
}
