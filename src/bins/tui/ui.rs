use std::fmt::Display;

use color_eyre::Result;
use crossterm::event::{Event, KeyCode, KeyModifiers};
use poks::{
    game::{Action, GameSetup, GameState, World, show_hand},
    player::{PlayerBehavior, PlayerLocal},
};
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
    message: Option<String>,
}

impl PoksTUI {
    pub fn new() -> Self {
        Self {
            world: World::new(4, poks::game::GameSetup::LocalAgainstCPU),
            should_exit: false,
            frame: 0,
            gamestate: GameState::Pause,
            message: None,
        }
    }

    pub fn should_exit(&self) -> bool {
        self.should_exit
    }

    pub fn update(&mut self) -> Result<()> {
        self.frame += 1;
        self.gamestate = self.world.tick_game()?;
        if self.gamestate == GameState::Finished {
            let winner = self.world.game.winner().expect("no winner despite win");
            self.message = Some("Game finished. Press F6 for a new game.".to_string());
        }

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
                KeyCode::F(1) => PlayerLocal::set_action(Action::Fold),
                KeyCode::F(2) => PlayerLocal::set_action(Action::Check),
                KeyCode::F(3) => PlayerLocal::set_action(Action::Raise(10)),
                KeyCode::F(4) => PlayerLocal::set_action(Action::Raise(50)),
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
        let mut buf = format!("Frame: {}", self.frame);
        if self.message.is_some() {
            let add = format!(" | Message: {}", self.message.as_ref().unwrap());
            buf.push_str(&add);
        }
        buf
    }

    fn controls(&self) -> String {
        "F1: Fold, F2: Check, F3: Raise, F4: All in".to_string()
    }

    fn world(&self) -> &World {
        &self.world
    }

    fn start_new_game(&mut self) {
        self.message = None;
        self.world.start_new_game();
    }

    fn gamedata(&self) -> String {
        let game = &self.world().game;
        format!(
            "Phase: {} | Turn of Player: {} | You are Player: {} | Pot: {} | Currency: {}€",
            game.phase(),
            game.turn,
            0,
            game.pot(),
            *self.world.players[0].currency()
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
        for (pid, action) in ac.iter() {
            if let Some(pid) = pid {
                buf.push_str(&format!("Player {pid}: {action}"));
            } else {
                buf.push_str(&action.to_string());
            }
            buf.push('\n');
        }
        buf
    }
}

fn line_widget<'a>(text: impl Display, borders: Borders, center: bool) -> Paragraph<'a> {
    let p = Paragraph::new(text.to_string()).block(Block::new().borders(borders));
    if center { p.centered() } else { p }
}
