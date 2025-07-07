use poks::game::evaluator;
use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Paragraph, Wrap},
};
use std::fmt::Display;

use crate::ui::{InputMode, PoksTUI};

impl PoksTUI {
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
        let mut buf = format!(
            "Mode: {:<10} | F1: Fold | F2: Check/Call | F3: Raise | F4: All in",
            self.input_mode
        );
        if self.bet.is_some() && self.input_mode == InputMode::Bet {
            buf.push_str(&format!(" | Bet: {}", self.bet.unwrap()));
        }
        buf
    }

    fn gamedata(&self) -> String {
        let game = &self.lobby().game;
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

    fn render_players(&self, area: Rect, frame: &mut Frame<'_>) {
        let players = self.lobby().players();
        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![Constraint::Min(5); players.len()])
            .split(area);

        for (idx, (player, layout)) in players.iter().zip(layout.iter()).enumerate() {
            frame.render_widget(
                Paragraph::new(format!(
                    "\n  Currency: {}\n  Total Bet: {}",
                    player.currency(),
                    self.lobby().game.players()[idx].total_bet()
                ))
                .block(Block::new().borders(Borders::ALL).title({
                    let mut pbuf = format!("Player {idx}");
                    if idx == self.player_id {
                        pbuf.push_str(" (You)");
                    }
                    if idx == self.lobby().game.big_blind_position() {
                        pbuf.push_str(" (BB)");
                    }
                    if idx == self.lobby().game.small_blind_position() {
                        pbuf.push_str(" (SB)");
                    }
                    if idx == self.lobby().game.dealer_position() {
                        pbuf.push_str(" (D)");
                    }
                    pbuf
                }))
                .wrap(Wrap { trim: false }),
                *layout,
            );
        }
    }

    fn render_world(&self, area: Rect, frame: &mut Frame<'_>) {
        let world = self.lobby();
        debug_assert!(!world.players().is_empty());

        let you = &world.game.players()[self.player_id];

        let panels = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(vec![
                Constraint::Length(2),
                Constraint::Length(35),
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

        self.render_players(panels[1], frame);
        self.render_action_log(panels[3], frame);
        frame.render_widget(
            line_widget(world.game.show_table(), Borders::ALL, true),
            layout_table[1],
        );
        frame.render_widget(
            line_widget(you.show_hand(), Borders::NONE, true),
            layout_phand[1],
        );
    }

    fn render_action_log(&self, area: Rect, frame: &mut Frame<'_>) {
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

        frame.render_widget(
            Paragraph::new(buf)
                .block(Block::new().borders(Borders::ALL))
                .wrap(Wrap { trim: false }),
            area,
        );
    }
}

impl Display for InputMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}

fn line_widget<'a>(text: impl Display, borders: Borders, center: bool) -> Paragraph<'a> {
    let p = Paragraph::new(text.to_string()).block(Block::new().borders(borders));
    if center { p.centered() } else { p }
}
