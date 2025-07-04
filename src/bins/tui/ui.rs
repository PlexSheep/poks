use color_eyre::Result;
use crossterm::event::{Event, KeyCode, KeyModifiers};
use poks::game::{Game, PlayerBehavior, World};
use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Paragraph},
};

struct GameWidget<'a>(Option<&'a Game>);

pub struct PoksTUI {
    world: World,
    should_exit: bool,
    frame: u32,
}

impl PoksTUI {
    pub fn new() -> Self {
        Self {
            world: World::new(4),
            should_exit: false,
            frame: 0,
        }
    }

    pub fn should_exit(&self) -> bool {
        self.should_exit
    }

    pub fn update(&mut self) -> Result<()> {
        self.frame += 1;
        if self.world().current_game.is_some() {
            self.world.tick_game();
        }
        Ok(())
    }

    #[allow(clippy::single_match)] // many possible events
    pub fn handle_event(&mut self, event: Event) -> Result<()> {
        match event {
            Event::Key(key) => match key.code {
                KeyCode::Char('q') => self.should_exit = true,
                KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                    self.should_exit = true
                }
                KeyCode::F(6) => self.start_new_game(),
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
                Constraint::Length(3),
            ])
            .split(frame.area());

        frame.render_widget(
            Paragraph::new("Top").block(Block::new().borders(Borders::ALL)),
            layout[0],
        );
        frame.render_widget(self.game_widget(), layout[1]);
        frame.render_widget(
            Paragraph::new(self.metadata()).block(Block::new().borders(Borders::ALL)),
            layout[2],
        );
    }

    fn metadata(&self) -> String {
        format!("Frame: {}", self.frame)
    }

    fn world(&self) -> &World {
        &self.world
    }

    fn world_mut(&mut self) -> &mut World {
        &mut self.world
    }

    fn game_widget(&self) -> GameWidget {
        GameWidget(self.world().current_game.as_ref())
    }

    fn start_new_game(&mut self) {
        self.world.start_new_game();
    }
}

impl GameWidget<'_> {
    fn render_nogame(&self, area: Rect, buf: &mut Buffer) {
        buf.set_string(
            area.left(),
            area.top(),
            "No game started yet. Start a new game with <F6>",
            Style::default(),
        );
    }
}

impl Widget for GameWidget<'_> {
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        let maybe_game = self.0;

        if maybe_game.is_none() {
            return self.render_nogame(area, buf);
        }

        let game = maybe_game.unwrap();
        debug_assert!(!game.players.is_empty());

        let you = game.players[0];

        buf.set_string(
            area.left(),
            area.top(),
            format!("{:?}", you.hand()),
            Style::default(),
        );
    }
}
