use std::fmt::Display;

use color_eyre::Result;
use crossterm::event::{Event, KeyCode, KeyModifiers};
use poks::{
    CU,
    game::{Action, PlayerID, evaluator, show_cards, show_eval_cards},
    player::PlayerCPU,
    world::World,
};
use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Paragraph, Wrap},
};
use tracing::info;

use crate::player::local::{ActionAccessor, PlayerLocal};

pub struct PoksTUI {
    world: World,
    should_exit: bool,
    frame: u32,
    message: Option<String>,
    player_af: ActionAccessor,
    player_id: PlayerID,
}

impl PoksTUI {
    pub fn new() -> Self {
        let mut worldb = World::builder();

        let player = Box::new(PlayerLocal::new());
        let player_action_field = player.action_field_reference();
        worldb.add_player(player).unwrap();

        for _ in 1..4 {
            worldb
                .add_player(Box::new(PlayerCPU::default()))
                .expect("could not add cpu player");
        }

        for player in worldb.players.iter_mut() {
            player.set_currency(CU!(5000));
        }

        Self {
            world: worldb.build().expect("could not prepare world"),
            should_exit: false,
            frame: 0,
            message: None,
            player_af: player_action_field,
            player_id: 0,
        }
    }

    pub fn should_exit(&self) -> bool {
        self.should_exit
    }

    pub fn update(&mut self) -> Result<()> {
        self.frame += 1;
        if self.world().game.is_finished() {
            self.message = Some("Game finished. Press F6 or Space for a new game.".to_string());
        } else {
            self.world.tick_game()?;
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
                KeyCode::F(6) | KeyCode::Char(' ') if self.world().game.is_finished() => {
                    self.start_new_game()
                }
                KeyCode::F(1) => PlayerLocal::set_action(&self.player_af, Action::Fold),
                // TODO: call needs calculation of diff
                KeyCode::F(2) => {
                    PlayerLocal::set_action(&self.player_af, self.world().game.action_call())
                }
                KeyCode::F(3) => PlayerLocal::set_action(&self.player_af, Action::Raise(CU!(10))),
                KeyCode::F(4) => PlayerLocal::set_action(&self.player_af, Action::Raise(CU!(50))),
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
        "F1: Fold, F2: Check/Call, F3: Raise, F4: All in".to_string()
    }

    fn world(&self) -> &World {
        &self.world
    }

    fn start_new_game(&mut self) {
        self.message = None;
        self.world
            .start_new_game()
            .expect("could not start new game");
    }

    fn gamedata(&self) -> String {
        let game = &self.world().game;
        let player = &self.world.players()[self.player_id];
        let mut buf = format!(
            "Turn of Player: {:01} | You are Player: {:01} | Pot: {} | Currency: {}",
            game.turn(),
            0,
            game.pot(),
            player.currency(),
        );

        if player.hand().is_some() && game.community_cards().len() >= 3 {
            let combined = game.hand_plus_table(self.player_id);

            let eval = evaluator()
                .evaluate_five(&*combined)
                .expect("could not evaluate player hand + community cards");
            buf.push_str(&format!(" | Evaluation: {eval}"));
        }

        buf
    }

    fn render_world(&self, area: Rect, frame: &mut Frame<'_>) {
        let world = self.world();
        debug_assert!(!world.players().is_empty());

        let you = &world.game.players()[self.player_id];

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
            {
                let text = self.render_action_log();
                let borders = Borders::ALL;
                Paragraph::new(text.to_string())
                    .block(Block::new().borders(borders))
                    .wrap(Wrap { trim: false })
            },
            panels[3],
        );
        frame.render_widget(
            line_widget(world.game.show_table(), Borders::ALL, true),
            layout_table[1],
        );
        frame.render_widget(
            line_widget(you.show_hand(), Borders::NONE, true),
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
