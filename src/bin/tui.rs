//! TUI Goldfish Mode — ratatui-based terminal UI for goldfish games.
//!
//! Play a goldfish game with a visual terminal interface. Navigate zones with
//! WASD, select cards/actions with Space, and watch the game state update
//! in real time.
//!
//! Usage:
//!   cargo run --release --features tui --bin tui
//!   cargo run --release --features tui --bin tui -- --preset kinnan
//!   cargo run --release --features tui --bin tui -- --deck decks/kinnan.txt
//!
//! Controls:
//!   W/S     Move selection up/down within a zone or action list
//!   A/D     Move selection left/right between zones or cards
//!   Space   Activate selected card / confirm action
//!   Escape  Cancel current action / deselect
//!   U       Undo last action
//!   Q       Quit

use std::collections::HashMap;
use std::io::{self, stdout};
use std::sync::Arc;

use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use ratatui::{
    prelude::*,
    widgets::*,
};

use mtg_gto::action::{legal_actions, Action};
use mtg_gto::card::{sample, CardDef, CardType, ObjectId};
use mtg_gto::game::{CardDatabase, GameState, Target};
use mtg_gto::rules;
use mtg_gto::strategy::{GoldfishStrategy, Strategy};

// ---------------------------------------------------------------------------
// Zone navigation model
// ---------------------------------------------------------------------------

/// The zones the player can navigate between.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum Zone {
    Hand,
    Battlefield,
    CommandZone,
    Graveyard,
    Exile,
    Stack,
    Actions,
}

const ZONE_ORDER: &[Zone] = &[
    Zone::Hand,
    Zone::Battlefield,
    Zone::CommandZone,
    Zone::Stack,
    Zone::Graveyard,
    Zone::Exile,
    Zone::Actions,
];

/// The current UI interaction mode.
#[derive(Debug, Clone, PartialEq, Eq)]
enum UiMode {
    /// Normal browsing — navigate zones and cards.
    Browse,
    /// Picking an action from the action list.
    ActionSelect,
    /// Game is over.
    GameOver,
}

/// TUI application state.
struct App {
    state: GameState,
    db: CardDatabase,
    goldfish: GoldfishStrategy,
    actions_taken: u32,
    undo_stack: Vec<GameState>,
    action_log: Vec<String>,

    // UI state
    mode: UiMode,
    active_zone: Zone,
    /// Per-zone selection index.
    zone_cursors: HashMap<Zone, usize>,
    /// Cached legal actions for current game state.
    cached_actions: Vec<Action>,
    /// Log messages shown in the status area.
    status_log: Vec<String>,

    should_quit: bool,
}

const MAX_TURNS: u32 = 20;
const MAX_ACTIONS: u32 = 10_000;

impl App {
    fn new(state: GameState, db: CardDatabase) -> Self {
        let cached_actions = legal_actions(&state);
        let mut zone_cursors = HashMap::new();
        for &z in ZONE_ORDER {
            zone_cursors.insert(z, 0);
        }
        App {
            state,
            db,
            goldfish: GoldfishStrategy,
            actions_taken: 0,
            undo_stack: Vec::new(),
            action_log: Vec::new(),

            mode: UiMode::Browse,
            active_zone: Zone::Hand,
            zone_cursors,
            cached_actions,
            status_log: vec!["Game started. WASD to navigate, Space to act, Q to quit.".into()],

            should_quit: false,
        }
    }

    /// Number of items in the current zone.
    fn zone_len(&self, zone: Zone) -> usize {
        match zone {
            Zone::Hand => self.state.players[0].hand.len(),
            Zone::Battlefield => self.state.permanents_controlled_by(0).len(),
            Zone::CommandZone => self.state.players[0].command_zone.len(),
            Zone::Graveyard => self.state.players[0].graveyard.len(),
            Zone::Exile => self.state.players[0].exile.len(),
            Zone::Stack => self.state.stack.len(),
            Zone::Actions => self.cached_actions.len(),
        }
    }

    fn cursor(&self) -> usize {
        *self.zone_cursors.get(&self.active_zone).unwrap_or(&0)
    }

    fn set_cursor(&mut self, val: usize) {
        let len = self.zone_len(self.active_zone);
        let clamped = if len == 0 { 0 } else { val.min(len - 1) };
        self.zone_cursors.insert(self.active_zone, clamped);
    }

    fn refresh_actions(&mut self) {
        self.cached_actions = legal_actions(&self.state);
        // Clamp action cursor
        let len = self.cached_actions.len();
        if len > 0 {
            let cur = *self.zone_cursors.get(&Zone::Actions).unwrap_or(&0);
            self.zone_cursors.insert(Zone::Actions, cur.min(len - 1));
        } else {
            self.zone_cursors.insert(Zone::Actions, 0);
        }
    }

    /// Auto-advance: handle goldfish turns and auto-pass situations.
    fn auto_advance(&mut self) {
        let mut passes = 0;
        while !self.state.game_over
            && self.state.turn_number <= MAX_TURNS
            && self.actions_taken < MAX_ACTIONS
            && passes < 200
        {
            let player = self.state.priority_player;
            let actions = legal_actions(&self.state);

            // Auto-pass if no meaningful choices
            if actions.is_empty()
                || (actions.len() == 1 && actions[0] == Action::PassPriority)
            {
                rules::apply_action(&mut self.state, &Action::PassPriority);
                self.actions_taken += 1;
                passes += 1;
                continue;
            }

            // If it's the goldfish's turn, auto-play
            if player != 0 {
                let action = self.goldfish.choose_action(&self.state, player);
                rules::apply_action(&mut self.state, &action);
                self.actions_taken += 1;
                passes += 1;
                continue;
            }

            // Player 0's turn with meaningful choices — stop
            break;
        }

        self.refresh_actions();

        if self.state.game_over {
            self.mode = UiMode::GameOver;
            let msg = match self.state.winner {
                Some(0) => format!("YOU WIN on turn {}!", self.state.turn_number),
                Some(_) => format!("You lost on turn {}.", self.state.turn_number),
                None => "Draw (turn limit reached).".into(),
            };
            self.status_log.push(msg);
        }
    }

    /// Execute the chosen action index.
    fn execute_action(&mut self, idx: usize) {
        if idx >= self.cached_actions.len() {
            return;
        }

        let action = self.cached_actions[idx].clone();
        let desc = format_action_rich(&self.state, &action, &self.db);

        // Save for undo
        self.undo_stack.push(self.state.clone());

        self.action_log.push(format!(
            "T{} {:?}: {}",
            self.state.turn_number, self.state.phase, desc
        ));

        self.status_log.push(format!("> {}", desc));

        rules::apply_action(&mut self.state, &action);
        self.actions_taken += 1;

        // Check SBAs
        rules::check_state_based_actions(&mut self.state);

        // Auto-advance through goldfish turns and forced passes
        self.auto_advance();

        self.mode = UiMode::Browse;
    }

    fn undo(&mut self) {
        if let Some(prev) = self.undo_stack.pop() {
            self.state = prev;
            if let Some(last) = self.action_log.pop() {
                self.status_log.push(format!("Undid: {}", last));
            }
            self.actions_taken = self.actions_taken.saturating_sub(1);
            self.auto_advance();
        } else {
            self.status_log.push("Nothing to undo.".into());
        }
    }

    /// Find actions relevant to the currently selected object in the current zone.
    fn actions_for_selected(&self) -> Vec<usize> {
        let zone = self.active_zone;
        let cursor = self.cursor();

        let obj_id: Option<ObjectId> = match zone {
            Zone::Hand => self.state.players[0].hand.get(cursor).copied(),
            Zone::Battlefield => {
                let perms = self.state.permanents_controlled_by(0);
                perms.get(cursor).copied()
            }
            Zone::CommandZone => self.state.players[0].command_zone.get(cursor).copied(),
            Zone::Graveyard => self.state.players[0].graveyard.get(cursor).copied(),
            _ => None,
        };

        let Some(oid) = obj_id else {
            return Vec::new();
        };

        self.cached_actions
            .iter()
            .enumerate()
            .filter(|(_, a)| action_involves_object(a, oid))
            .map(|(i, _)| i)
            .collect()
    }

    fn handle_key(&mut self, code: KeyCode) {
        match self.mode {
            UiMode::GameOver => {
                if code == KeyCode::Char('q') || code == KeyCode::Char('Q') {
                    self.should_quit = true;
                }
            }
            UiMode::ActionSelect => match code {
                KeyCode::Char('w') | KeyCode::Char('W') | KeyCode::Up => {
                    let cur = self.cursor();
                    if cur > 0 {
                        self.set_cursor(cur - 1);
                    }
                }
                KeyCode::Char('s') | KeyCode::Char('S') | KeyCode::Down => {
                    let cur = self.cursor();
                    self.set_cursor(cur + 1);
                }
                KeyCode::Char(' ') | KeyCode::Enter => {
                    let idx = self.cursor();
                    self.execute_action(idx);
                }
                KeyCode::Esc => {
                    self.mode = UiMode::Browse;
                    self.active_zone = Zone::Hand;
                }
                KeyCode::Char('q') | KeyCode::Char('Q') => {
                    self.should_quit = true;
                }
                _ => {}
            },
            UiMode::Browse => match code {
                KeyCode::Char('w') | KeyCode::Char('W') | KeyCode::Up => {
                    // Move to previous zone
                    let idx = ZONE_ORDER.iter().position(|&z| z == self.active_zone).unwrap_or(0);
                    if idx > 0 {
                        self.active_zone = ZONE_ORDER[idx - 1];
                    }
                }
                KeyCode::Char('s') | KeyCode::Char('S') | KeyCode::Down => {
                    // Move to next zone
                    let idx = ZONE_ORDER.iter().position(|&z| z == self.active_zone).unwrap_or(0);
                    if idx + 1 < ZONE_ORDER.len() {
                        self.active_zone = ZONE_ORDER[idx + 1];
                    }
                }
                KeyCode::Char('a') | KeyCode::Char('A') | KeyCode::Left => {
                    let cur = self.cursor();
                    if cur > 0 {
                        self.set_cursor(cur - 1);
                    }
                }
                KeyCode::Char('d') | KeyCode::Char('D') | KeyCode::Right => {
                    let cur = self.cursor();
                    self.set_cursor(cur + 1);
                }
                KeyCode::Char(' ') | KeyCode::Enter => {
                    if self.active_zone == Zone::Actions {
                        // Direct action selection
                        let idx = self.cursor();
                        self.execute_action(idx);
                    } else {
                        // Find actions for selected card
                        let relevant = self.actions_for_selected();
                        if relevant.len() == 1 {
                            // Single action — execute immediately
                            self.execute_action(relevant[0]);
                        } else if relevant.len() > 1 {
                            // Multiple actions — switch to action select filtered
                            self.mode = UiMode::ActionSelect;
                            self.active_zone = Zone::Actions;
                            self.zone_cursors.insert(Zone::Actions, relevant[0]);
                        } else {
                            self.status_log.push("No actions available for this card.".into());
                        }
                    }
                }
                KeyCode::Char('u') | KeyCode::Char('U') => {
                    self.undo();
                }
                KeyCode::Char('q') | KeyCode::Char('Q') => {
                    self.should_quit = true;
                }
                KeyCode::Esc => {
                    // Deselect
                    self.zone_cursors.insert(self.active_zone, 0);
                }
                _ => {}
            },
        }
    }
}

/// Check if an action involves a specific object.
fn action_involves_object(action: &Action, oid: ObjectId) -> bool {
    match action {
        Action::PlayLand { object_id }
        | Action::CastSpell { object_id, .. }
        | Action::CastCommander { object_id, .. }
        | Action::ActivateManaAbility { object_id, .. }
        | Action::ActivateAbility { object_id, .. }
        | Action::ActivateLoyalty { object_id, .. }
        | Action::CastFromGraveyard { object_id, .. }
        | Action::Discard { object_id }
        | Action::MulliganBottomCard { object_id } => *object_id == oid,
        Action::DeclareAttackers { attackers } => attackers.contains(&oid),
        Action::Equip { equipment_id, target_id } => {
            *equipment_id == oid || *target_id == oid
        }
        _ => false,
    }
}

// ---------------------------------------------------------------------------
// Rendering
// ---------------------------------------------------------------------------

fn ui(f: &mut Frame, app: &App) {
    // Main layout: top bar, middle content, bottom status
    let outer = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),  // Header
            Constraint::Min(10),   // Main content
            Constraint::Length(6), // Status/log
        ])
        .split(f.area());

    render_header(f, app, outer[0]);
    render_main(f, app, outer[1]);
    render_status(f, app, outer[2]);
}

fn render_header(f: &mut Frame, app: &App, area: Rect) {
    let phase_str = format!("{:?}", app.state.phase);
    let pool = &app.state.players[0].mana_pool;
    let mana_str = format_mana_pool(pool);

    let opp_life = app.state.players[1].life;

    let header_text = format!(
        " Turn {} | Phase: {} | Life: {} | Opp Life: {} | Mana: {} | Actions: {} ",
        app.state.turn_number,
        phase_str,
        app.state.players[0].life,
        opp_life,
        if mana_str.is_empty() { "empty".into() } else { mana_str },
        app.actions_taken,
    );

    let mode_str = match &app.mode {
        UiMode::Browse => "[BROWSE]",
        UiMode::ActionSelect => "[SELECT ACTION]",
        UiMode::GameOver => "[GAME OVER]",
    };

    let block = Block::default()
        .title(format!(" MTG Goldfish TUI {} ", mode_str))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));

    let text = Paragraph::new(header_text)
        .block(block)
        .style(Style::default().fg(Color::White));

    f.render_widget(text, area);
}

fn render_main(f: &mut Frame, app: &App, area: Rect) {
    // Split into left (zones) and right (actions)
    let main_cols = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(60), // Zones
            Constraint::Percentage(40), // Actions
        ])
        .split(area);

    render_zones(f, app, main_cols[0]);
    render_actions(f, app, main_cols[1]);
}

fn render_zones(f: &mut Frame, app: &App, area: Rect) {
    // Stack zones vertically: Hand, Battlefield, Command, Graveyard, Exile, Stack
    let zone_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(4),    // Hand
            Constraint::Min(4),    // Battlefield
            Constraint::Length(3), // Command Zone
            Constraint::Length(3), // Stack
            Constraint::Length(3), // Graveyard
            Constraint::Length(3), // Exile
        ])
        .split(area);

    render_zone_hand(f, app, zone_layout[0]);
    render_zone_battlefield(f, app, zone_layout[1]);
    render_zone_command(f, app, zone_layout[2]);
    render_zone_stack(f, app, zone_layout[3]);
    render_zone_graveyard(f, app, zone_layout[4]);
    render_zone_exile(f, app, zone_layout[5]);
}

fn zone_border_style(app: &App, zone: Zone) -> Style {
    if app.active_zone == zone {
        Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::DarkGray)
    }
}

fn render_zone_hand(f: &mut Frame, app: &App, area: Rect) {
    let hand = &app.state.players[0].hand;
    let cursor = if app.active_zone == Zone::Hand { Some(app.cursor()) } else { None };

    let items: Vec<Line> = hand.iter().enumerate().map(|(i, &oid)| {
        let text = format_card_in_hand(&app.state, &app.db, oid);
        let style = if cursor == Some(i) {
            Style::default().fg(Color::Black).bg(Color::Yellow).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::White)
        };
        Line::styled(text, style)
    }).collect();

    let block = Block::default()
        .title(format!(" Hand ({}) ", hand.len()))
        .borders(Borders::ALL)
        .border_style(zone_border_style(app, Zone::Hand));

    let para = Paragraph::new(items)
        .block(block)
        .wrap(Wrap { trim: false });

    f.render_widget(para, area);
}

fn render_zone_battlefield(f: &mut Frame, app: &App, area: Rect) {
    let perms = app.state.permanents_controlled_by(0);
    let cursor = if app.active_zone == Zone::Battlefield { Some(app.cursor()) } else { None };

    let items: Vec<Line> = perms.iter().enumerate().map(|(i, &oid)| {
        let text = format_card_on_battlefield(&app.state, &app.db, oid);
        let style = if cursor == Some(i) {
            Style::default().fg(Color::Black).bg(Color::Yellow).add_modifier(Modifier::BOLD)
        } else {
            let inst = &app.state.objects[&oid];
            if inst.tapped {
                Style::default().fg(Color::DarkGray)
            } else {
                Style::default().fg(Color::Green)
            }
        };
        Line::styled(text, style)
    }).collect();

    let block = Block::default()
        .title(format!(" Battlefield ({}) ", perms.len()))
        .borders(Borders::ALL)
        .border_style(zone_border_style(app, Zone::Battlefield));

    let para = Paragraph::new(items)
        .block(block)
        .wrap(Wrap { trim: false });

    f.render_widget(para, area);
}

fn render_zone_command(f: &mut Frame, app: &App, area: Rect) {
    let cz = &app.state.players[0].command_zone;
    let cursor = if app.active_zone == Zone::CommandZone { Some(app.cursor()) } else { None };

    let items: Vec<Line> = cz.iter().enumerate().map(|(i, &oid)| {
        let name = card_name(&app.state, oid, &app.db);
        let tax = app.state.players[0].commander_tax;
        let text = if tax > 0 {
            format!("{} (tax: {})", name, tax * 2)
        } else {
            name
        };
        let style = if cursor == Some(i) {
            Style::default().fg(Color::Black).bg(Color::Yellow).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::Magenta)
        };
        Line::styled(text, style)
    }).collect();

    let block = Block::default()
        .title(format!(" Command Zone ({}) ", cz.len()))
        .borders(Borders::ALL)
        .border_style(zone_border_style(app, Zone::CommandZone));

    let para = Paragraph::new(items)
        .block(block)
        .wrap(Wrap { trim: false });

    f.render_widget(para, area);
}

fn render_zone_stack(f: &mut Frame, app: &App, area: Rect) {
    let stack = &app.state.stack;
    let cursor = if app.active_zone == Zone::Stack { Some(app.cursor()) } else { None };

    let items: Vec<Line> = stack.iter().enumerate().rev().map(|(i, entry)| {
        let desc = format_stack_entry(&app.state, entry, &app.db);
        let style = if cursor == Some(i) {
            Style::default().fg(Color::Black).bg(Color::Yellow).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::Red)
        };
        Line::styled(desc, style)
    }).collect();

    let block = Block::default()
        .title(format!(" Stack ({}) ", stack.len()))
        .borders(Borders::ALL)
        .border_style(zone_border_style(app, Zone::Stack));

    let para = Paragraph::new(items)
        .block(block)
        .wrap(Wrap { trim: false });

    f.render_widget(para, area);
}

fn render_zone_graveyard(f: &mut Frame, app: &App, area: Rect) {
    let gy = &app.state.players[0].graveyard;
    let cursor = if app.active_zone == Zone::Graveyard { Some(app.cursor()) } else { None };

    let items: Vec<Line> = gy.iter().enumerate().map(|(i, &oid)| {
        let name = card_name(&app.state, oid, &app.db);
        let style = if cursor == Some(i) {
            Style::default().fg(Color::Black).bg(Color::Yellow).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::DarkGray)
        };
        Line::styled(name, style)
    }).collect();

    let block = Block::default()
        .title(format!(" Graveyard ({}) ", gy.len()))
        .borders(Borders::ALL)
        .border_style(zone_border_style(app, Zone::Graveyard));

    let para = Paragraph::new(items)
        .block(block)
        .wrap(Wrap { trim: false });

    f.render_widget(para, area);
}

fn render_zone_exile(f: &mut Frame, app: &App, area: Rect) {
    let ex = &app.state.players[0].exile;
    let cursor = if app.active_zone == Zone::Exile { Some(app.cursor()) } else { None };

    let items: Vec<Line> = ex.iter().enumerate().map(|(i, &oid)| {
        let name = card_name(&app.state, oid, &app.db);
        let style = if cursor == Some(i) {
            Style::default().fg(Color::Black).bg(Color::Yellow).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::Gray)
        };
        Line::styled(name, style)
    }).collect();

    let block = Block::default()
        .title(format!(" Exile ({}) ", ex.len()))
        .borders(Borders::ALL)
        .border_style(zone_border_style(app, Zone::Exile));

    let para = Paragraph::new(items)
        .block(block)
        .wrap(Wrap { trim: false });

    f.render_widget(para, area);
}

fn render_actions(f: &mut Frame, app: &App, area: Rect) {
    // Split into actions list and card detail
    let action_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(6),    // Actions list
            Constraint::Length(8), // Card detail / library info
        ])
        .split(area);

    // Actions list
    let cursor = if app.active_zone == Zone::Actions || app.mode == UiMode::ActionSelect {
        Some(*app.zone_cursors.get(&Zone::Actions).unwrap_or(&0))
    } else {
        None
    };

    let items: Vec<Line> = app.cached_actions.iter().enumerate().map(|(i, action)| {
        let desc = format_action_short(&app.state, action, &app.db);
        let style = if cursor == Some(i) {
            Style::default().fg(Color::Black).bg(Color::Cyan).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::White)
        };
        Line::styled(format!(" {} {}", if cursor == Some(i) { ">" } else { " " }, desc), style)
    }).collect();

    let action_border = if app.active_zone == Zone::Actions || app.mode == UiMode::ActionSelect {
        Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::DarkGray)
    };

    let block = Block::default()
        .title(format!(" Actions ({}) ", app.cached_actions.len()))
        .borders(Borders::ALL)
        .border_style(action_border);

    let para = Paragraph::new(items)
        .block(block)
        .wrap(Wrap { trim: false })
        .scroll((cursor.unwrap_or(0).saturating_sub(4) as u16, 0));

    f.render_widget(para, action_layout[0]);

    // Card detail / info panel
    render_detail_panel(f, app, action_layout[1]);
}

fn render_detail_panel(f: &mut Frame, app: &App, area: Rect) {
    let mut lines: Vec<Line> = Vec::new();

    // Show selected card details
    let selected_oid = get_selected_object(app);
    if let Some(oid) = selected_oid {
        if let Some(inst) = app.state.objects.get(&oid) {
            if let Some(def) = app.db.get(inst.card_def_id) {
                lines.push(Line::styled(
                    def.name.clone(),
                    Style::default().fg(Color::White).add_modifier(Modifier::BOLD),
                ));
                let cost_str = def.mana_cost.as_ref()
                    .map(|c| format!("{}", c))
                    .unwrap_or_else(|| "—".into());
                lines.push(Line::styled(
                    format!("{} | {}", format_card_type(def), cost_str),
                    Style::default().fg(Color::Gray),
                ));
                if def.is_creature() {
                    let p = app.state.effective_power(oid);
                    let t = app.state.effective_toughness(oid);
                    lines.push(Line::styled(
                        format!("P/T: {}/{}", p, t),
                        Style::default().fg(Color::Green),
                    ));
                }
                if !def.oracle_text.is_empty() {
                    // Truncate oracle text to fit
                    let text = if def.oracle_text.len() > 80 {
                        format!("{}...", &def.oracle_text[..77])
                    } else {
                        def.oracle_text.clone()
                    };
                    lines.push(Line::styled(text, Style::default().fg(Color::DarkGray)));
                }
            }
        }
    }

    // Library count
    lines.push(Line::styled(
        format!("Library: {} cards", app.state.players[0].library.len()),
        Style::default().fg(Color::Blue),
    ));

    let block = Block::default()
        .title(" Detail ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::DarkGray));

    let para = Paragraph::new(lines)
        .block(block)
        .wrap(Wrap { trim: false });

    f.render_widget(para, area);
}

fn render_status(f: &mut Frame, app: &App, area: Rect) {
    let block = Block::default()
        .title(" Log ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::DarkGray));

    // Show last N log messages
    let max_lines = area.height.saturating_sub(2) as usize;
    let start = app.status_log.len().saturating_sub(max_lines);
    let items: Vec<Line> = app.status_log[start..].iter().map(|s| {
        Line::styled(s.clone(), Style::default().fg(Color::Gray))
    }).collect();

    let para = Paragraph::new(items)
        .block(block)
        .wrap(Wrap { trim: false });

    f.render_widget(para, area);
}

// ---------------------------------------------------------------------------
// Get selected object
// ---------------------------------------------------------------------------

fn get_selected_object(app: &App) -> Option<ObjectId> {
    let cursor = app.cursor();
    match app.active_zone {
        Zone::Hand => app.state.players[0].hand.get(cursor).copied(),
        Zone::Battlefield => {
            let perms = app.state.permanents_controlled_by(0);
            perms.get(cursor).copied()
        }
        Zone::CommandZone => app.state.players[0].command_zone.get(cursor).copied(),
        Zone::Graveyard => app.state.players[0].graveyard.get(cursor).copied(),
        Zone::Exile => app.state.players[0].exile.get(cursor).copied(),
        _ => None,
    }
}

// ---------------------------------------------------------------------------
// Formatting helpers
// ---------------------------------------------------------------------------

fn card_name(state: &GameState, obj_id: ObjectId, db: &CardDatabase) -> String {
    state.objects.get(&obj_id)
        .and_then(|inst| db.get(inst.card_def_id))
        .map(|d| d.name.clone())
        .unwrap_or_else(|| format!("obj#{}", obj_id))
}

fn format_card_type(def: &CardDef) -> String {
    def.card_types.iter().map(|t| match t {
        CardType::Creature => "Creature",
        CardType::Instant => "Instant",
        CardType::Sorcery => "Sorcery",
        CardType::Enchantment => "Enchantment",
        CardType::Artifact => "Artifact",
        CardType::Land => "Land",
        CardType::Planeswalker => "Planeswalker",
    }).collect::<Vec<_>>().join(" ")
}

fn format_card_in_hand(state: &GameState, db: &CardDatabase, oid: ObjectId) -> String {
    let inst = &state.objects[&oid];
    let Some(def) = db.get(inst.card_def_id) else {
        return format!("obj#{}", oid);
    };

    let cost = def.mana_cost.as_ref()
        .map(|c| format!(" {}", c))
        .unwrap_or_default();

    let type_str = format_card_type(def);

    let stats = if def.is_creature() {
        format!(" {}/{}", def.power.unwrap_or(0), def.toughness.unwrap_or(0))
    } else {
        String::new()
    };

    format!("{}{} - {}{}", def.name, cost, type_str, stats)
}

fn format_card_on_battlefield(state: &GameState, db: &CardDatabase, oid: ObjectId) -> String {
    let inst = &state.objects[&oid];
    let Some(def) = db.get(inst.card_def_id) else {
        return format!("obj#{}", oid);
    };

    let mut parts = vec![def.name.clone()];

    if def.is_creature() {
        let p = state.effective_power(oid);
        let t = state.effective_toughness(oid);
        parts.push(format!("{}/{}", p, t));
        if inst.damage_marked > 0 {
            parts.push(format!("[{} dmg]", inst.damage_marked));
        }
        if inst.plus_counters != 0 {
            parts.push(format!("+{} counters", inst.plus_counters));
        }
    }

    if inst.tapped {
        parts.push("(T)".into());
    }
    if inst.summoning_sick && def.is_creature() {
        parts.push("(sick)".into());
    }

    parts.join(" ")
}

fn format_stack_entry(state: &GameState, entry: &mtg_gto::game::StackEntry, db: &CardDatabase) -> String {
    match &entry.source {
        mtg_gto::game::StackSource::Spell(obj_id) => {
            let name = card_name(state, *obj_id, db);
            format!("Spell: {}", name)
        }
        mtg_gto::game::StackSource::ActivatedAbility { source_id, ability_index } => {
            let name = card_name(state, *source_id, db);
            format!("Ability: {} #{}", name, ability_index)
        }
        mtg_gto::game::StackSource::TriggeredAbility { source_id, ability_index } => {
            let name = card_name(state, *source_id, db);
            format!("Trigger: {} #{}", name, ability_index)
        }
    }
}

fn format_mana_pool(pool: &mtg_gto::mana::ManaPool) -> String {
    let mut parts = Vec::new();
    if pool.white > 0 { parts.push(format!("{}W", pool.white)); }
    if pool.blue > 0 { parts.push(format!("{}U", pool.blue)); }
    if pool.black > 0 { parts.push(format!("{}B", pool.black)); }
    if pool.red > 0 { parts.push(format!("{}R", pool.red)); }
    if pool.green > 0 { parts.push(format!("{}G", pool.green)); }
    if pool.colorless > 0 { parts.push(format!("{}C", pool.colorless)); }
    parts.join(" ")
}

fn format_action_short(state: &GameState, action: &Action, db: &CardDatabase) -> String {
    match action {
        Action::PassPriority => "Pass priority".into(),
        Action::PlayLand { object_id } => {
            format!("Play land: {}", card_name(state, *object_id, db))
        }
        Action::CastSpell { object_id, targets } => {
            let name = card_name(state, *object_id, db);
            let tgt = format_targets(state, targets, db);
            if tgt.is_empty() {
                format!("Cast: {}", name)
            } else {
                format!("Cast: {} -> {}", name, tgt)
            }
        }
        Action::CastCommander { object_id, targets } => {
            let name = card_name(state, *object_id, db);
            let tgt = format_targets(state, targets, db);
            if tgt.is_empty() {
                format!("Cast cmdr: {}", name)
            } else {
                format!("Cast cmdr: {} -> {}", name, tgt)
            }
        }
        Action::ActivateManaAbility { object_id, ability_index } => {
            let name = card_name(state, *object_id, db);
            let inst = &state.objects[object_id];
            let mana_desc = db.get(inst.card_def_id)
                .and_then(|d| d.mana_abilities.get(*ability_index))
                .map(|ma| format!("{:?}", ma))
                .unwrap_or_else(|| "?".into());
            format!("Tap: {} ({})", name, mana_desc)
        }
        Action::ActivateAbility { object_id, ability_index, targets } => {
            let name = card_name(state, *object_id, db);
            let tgt = format_targets(state, targets, db);
            if tgt.is_empty() {
                format!("Activate: {} #{}", name, ability_index)
            } else {
                format!("Activate: {} #{} -> {}", name, ability_index, tgt)
            }
        }
        Action::DeclareAttackers { attackers } => {
            if attackers.is_empty() {
                "Attack: none".into()
            } else {
                let names: Vec<String> = attackers.iter().map(|&id| {
                    card_name(state, id, db)
                }).collect();
                format!("Attack: [{}]", names.join(", "))
            }
        }
        Action::DeclareBlockers { blocks } => {
            if blocks.is_empty() {
                "Block: none".into()
            } else {
                format!("Block with {} creatures", blocks.len())
            }
        }
        Action::Discard { object_id } => {
            format!("Discard: {}", card_name(state, *object_id, db))
        }
        Action::MulliganKeep => "Keep hand".into(),
        Action::MulliganMulligan => "Mulligan".into(),
        Action::MulliganBottomCard { object_id } => {
            format!("Bottom: {}", card_name(state, *object_id, db))
        }
        Action::ChooseTutorTarget { card_id } => {
            let name = db.get(*card_id)
                .map(|d| d.name.as_str())
                .unwrap_or("?");
            format!("Tutor: {}", name)
        }
        Action::ActivateLoyalty { object_id, ability_index } => {
            let name = card_name(state, *object_id, db);
            format!("Loyalty: {} #{}", name, ability_index)
        }
        Action::Equip { equipment_id, target_id } => {
            format!("Equip {} -> {}", card_name(state, *equipment_id, db), card_name(state, *target_id, db))
        }
        Action::CastFromGraveyard { object_id, .. } => {
            format!("Flashback: {}", card_name(state, *object_id, db))
        }
        Action::OrderTriggers { ordering } => format!("Order {} triggers", ordering.len()),
        Action::ChooseReplacementOrder { ordering } => format!("Order {} replacements", ordering.len()),
        Action::OrderDamageAssignment { .. } => "Assign damage".into(),
        Action::Concede => "Concede".into(),
        Action::ActivateMacro { combo_id } => format!("Combo #{}", combo_id),
    }
}

fn format_action_rich(state: &GameState, action: &Action, db: &CardDatabase) -> String {
    format_action_short(state, action, db)
}

fn format_targets(state: &GameState, targets: &[Target], db: &CardDatabase) -> String {
    if targets.is_empty() {
        return String::new();
    }
    let descs: Vec<String> = targets.iter().map(|t| match t {
        Target::Player(p) => {
            if *p == 0 { "You".into() } else { "Goldfish".into() }
        }
        Target::Object(id) => card_name(state, *id, db),
    }).collect();
    descs.join(", ")
}

// ---------------------------------------------------------------------------
// Deck loading (same as goldfish binary)
// ---------------------------------------------------------------------------

fn get_arg(args: &[String], flag: &str) -> Option<String> {
    args.iter()
        .position(|a| a == flag)
        .and_then(|i| args.get(i + 1).cloned())
}

fn load_game(args: &[String]) -> (GameState, CardDatabase) {
    let deck_path = get_arg(args, "--deck");
    let preset = get_arg(args, "--preset").unwrap_or_else(|| "kinnan".to_string());

    let db = sample::build_sample_db();

    let (deck, commander, is_commander) = if let Some(path) = deck_path {
        let path = std::path::Path::new(&path);
        match mtg_gto::deck_import::import_deck_from_file(path, &db) {
            Ok(decklist) => {
                let commander = if !decklist.commanders.is_empty() {
                    Some(decklist.commanders[0].card_id)
                } else {
                    None
                };
                let deck = decklist.expand();
                (deck, commander, commander.is_some())
            }
            Err(e) => {
                eprintln!("Error loading deck: {}", e);
                std::process::exit(1);
            }
        }
    } else {
        match preset.as_str() {
            "red" => (sample::red_aggro_deck(), None, false),
            "green" => (sample::green_stompy_deck(), None, false),
            "brimaz" => {
                let (d, c) = sample::brimaz_commander_deck();
                (d, Some(c), true)
            }
            "ashcoat" => {
                let (d, c) = sample::ashcoat_commander_deck();
                (d, Some(c), true)
            }
            _ => {
                let (d, c, _) = sample::kinnan_commander_deck();
                (d, Some(c), true)
            }
        }
    };

    let db_arc = Arc::new(db.clone());
    let state = if is_commander {
        let mut s = GameState::new_commander(2);
        s.card_db = Some(db_arc);
        let cmd = commander.unwrap();
        rules::setup_commander_game(&mut s, &deck, &deck, cmd, cmd);
        s
    } else {
        let mut s = GameState::new(2);
        s.card_db = Some(db_arc);
        rules::setup_game(&mut s, &deck, &deck);
        s
    };

    (state, db)
}

// ---------------------------------------------------------------------------
// Main
// ---------------------------------------------------------------------------

fn main() -> io::Result<()> {
    let args: Vec<String> = std::env::args().collect();

    let (state, db) = load_game(&args);
    let mut app = App::new(state, db);

    // Initial auto-advance (handle mulligan for goldfish, etc.)
    app.auto_advance();

    // Setup terminal
    enable_raw_mode()?;
    stdout().execute(EnterAlternateScreen)?;
    let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;

    // Main loop
    loop {
        terminal.draw(|f| ui(f, &app))?;

        if app.should_quit {
            break;
        }

        if let Event::Key(key) = event::read()? {
            if key.kind == KeyEventKind::Press {
                app.handle_key(key.code);
            }
        }
    }

    // Restore terminal
    disable_raw_mode()?;
    stdout().execute(LeaveAlternateScreen)?;

    // Print action log on exit
    if !app.action_log.is_empty() {
        println!("\n=== Action Log ({} actions) ===", app.action_log.len());
        for (i, entry) in app.action_log.iter().enumerate() {
            println!("  {:>3}. {}", i + 1, entry);
        }
    }

    if app.state.game_over {
        match app.state.winner {
            Some(0) => println!("\nYOU WIN on turn {}!", app.state.turn_number),
            Some(_) => println!("\nYou lost on turn {}.", app.state.turn_number),
            None => println!("\nDraw."),
        }
        println!(
            "Final life: You={} Goldfish={}",
            app.state.players[0].life, app.state.players[1].life
        );
    }

    Ok(())
}
