use color_eyre::Result;
use crossterm::event::{Event, KeyCode, KeyModifiers};
use poks::game::{Action, Game, GameState, PlayerBehavior, PlayerLocal, World};
use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Paragraph},
};
use tracing::info;

struct WorldWidget<'a>(&'a World);

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

        frame.render_widget(
            Paragraph::new(self.gamedata()).block(Block::new().borders(Borders::ALL)),
            layout[0],
        );
        frame.render_widget(self.world_widget(), layout[1]);
        frame.render_widget(
            Paragraph::new(self.metadata())
                .block(Block::new().borders(Borders::TOP | Borders::LEFT | Borders::RIGHT)),
            layout[2],
        );
        frame.render_widget(
            Paragraph::new(self.controls())
                .block(Block::new().borders(Borders::BOTTOM | Borders::LEFT | Borders::RIGHT)),
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

    fn world_widget(&self) -> WorldWidget {
        WorldWidget(self.world())
    }

    fn start_new_game(&mut self) {
        self.world.start_new_game();
    }

    fn gamedata(&self) -> String {
        let game = &self.world().game;
        format!(
            "Phase: {}, Turn of Player  {}, You are Player {}",
            game.phase(),
            game.turn,
            0
        )
    }
}

impl WorldWidget<'_> {}

impl Widget for WorldWidget<'_> {
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        let world = self.0;
        debug_assert!(!world.players.is_empty());

        let you = &world.players[0];

        buf.set_string(
            area.left(),
            area.top(),
            you.hand()
                .map(|h| h.to_string())
                .unwrap_or("(None)".to_string()),
            Style::default(),
        );
    }
}

fn set_player_action(action: Action) {
    PlayerLocal::set_action(action);
    PlayerLocal::set_action_is_ready(true);
}
