use std::fmt::Display;

use color_eyre::Result;
use crossterm::event::{Event, KeyCode, KeyModifiers};
use poks::game::{Action, GameState, PlayerBehavior, PlayerLocal, World, show_hand};
use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Paragraph},
};
use tracing::info;

pub struct PoksTUI {
    world: World,
    should_exit: bool,
    frame: u32,
    gamestate: GameState,
}

impl PoksTUI {
    pub fn new() -> Self {
        Self {
            world: World::new(4),
            should_exit: false,
            frame: 0,
            gamestate: GameState::Pause,
        }
    }

    pub fn should_exit(&self) -> bool {
        self.should_exit
    }

    pub fn update(&mut self) -> Result<()> {
        self.frame += 1;
        self.gamestate = self.world.tick_game()?;

        Ok(())
    }

    #[allow(clippy::single_match)] // many possible events
    pub fn handle_event(&mut self, event: Event) -> Result<()> {
        match event {
            Event::Key(key) => match key.code {
                KeyCode::Char('q') => {
                    info!("should exit");
                    self.should_exit = true
                }
                KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                    info!("should exit");
                    self.should_exit = true
                }
                KeyCode::F(6) => self.start_new_game(),
                KeyCode::F(1) => set_player_action(Action::Fold),
                KeyCode::F(2) => set_player_action(Action::Check),
                KeyCode::F(3) => set_player_action(Action::Raise(10)),
                KeyCode::F(4) => set_player_action(Action::Raise(50)),
                _ => (),
            },
            _ => (),
        }
        Ok(())
    }

    pub fn render(&self, frame: &mut ratatui::Frame<'_>) {
        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![
                Constraint::Length(3),
                Constraint::Min(10),
                Constraint::Length(2),
                Constraint::Length(2),
            ])
            .split(frame.area());

        frame.render_widget(line_widget(self.gamedata(), Borders::ALL, false), layout[0]);
        self.render_world(layout[1], frame);
        frame.render_widget(
            line_widget(
                self.metadata(),
                Borders::TOP | Borders::LEFT | Borders::RIGHT,
                false,
            ),
            layout[2],
        );
        frame.render_widget(
            line_widget(
                self.controls(),
                Borders::BOTTOM | Borders::LEFT | Borders::RIGHT,
                false,
            ),
            layout[3],
        );
    }

    fn metadata(&self) -> String {
        format!("Frame: {}", self.frame)
    }

    fn controls(&self) -> String {
        format!("F1: Fold, F2: Check, F3: Raise, F4: All in")
    }

    fn world(&self) -> &World {
        &self.world
    }

    fn world_mut(&mut self) -> &mut World {
        &mut self.world
    }

    fn start_new_game(&mut self) {
        self.world.start_new_game();
    }

    fn gamedata(&self) -> String {
        let game = &self.world().game;
        format!(
            "Phase: {}, Turn of Player: {}, You are Player: {}",
            game.phase(),
            game.turn,
            0
        )
    }

    fn render_world(&self, area: Rect, frame: &mut Frame<'_>) {
        let world = self.world();
        debug_assert!(!world.players.is_empty());

        let you = &world.players[0];

        let panels = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(vec![
                Constraint::Length(2),
                Constraint::Min(20),
                Constraint::Min(40),
                Constraint::Min(20),
                Constraint::Length(2),
            ])
            .split(area);
        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![
                Constraint::Min(2),
                Constraint::Length(3),
                Constraint::Min(2),
                Constraint::Length(3),
                Constraint::Length(1),
            ])
            .split(panels[2]);
        let layout_table = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(vec![
                Constraint::Min(1),
                Constraint::Length(36),
                Constraint::Min(1),
            ])
            .split(layout[1]);

        let layout_phand = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(vec![
                Constraint::Min(1),
                Constraint::Length(20),
                Constraint::Min(1),
            ])
            .split(layout[3]);

        frame.render_widget(
            line_widget(self.render_action_log(), Borders::ALL, false),
            panels[3],
        );
        frame.render_widget(
            line_widget(world.show_table(), Borders::ALL, true),
            layout_table[1],
        );
        frame.render_widget(
            line_widget(show_hand(*you.hand()), Borders::NONE, true),
            layout_phand[1],
        );
    }

    fn render_action_log(&self) -> String {
        let ac = self.world.action_log();
        let mut buf = String::with_capacity(ac.len() * 40);
        for line in ac
            .iter()
            .rev()
            .map(|(pid, action)| format!("Player {pid}: {action}"))
        {
            buf.push_str(&line);
            buf.push('\n');
        }
        buf
    }
}

fn set_player_action(action: Action) {
    PlayerLocal::set_action(action);
    PlayerLocal::set_action_is_ready(true);
}

fn line_widget<'a>(text: impl Display, borders: Borders, center: bool) -> Paragraph<'a> {
    let p = Paragraph::new(text.to_string()).block(Block::new().borders(borders));
    if center { p.centered() } else { p }
}
