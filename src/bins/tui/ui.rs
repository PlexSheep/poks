use color_eyre::Result;
use crossterm::event::{Event, KeyCode, KeyModifiers};
use poks::game::World;
use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Paragraph},
};

pub struct PoksTUI {
    world: World,
    should_exit: bool,
}

impl PoksTUI {
    pub fn new() -> Self {
        Self {
            world: World::new(4),
            should_exit: false,
        }
    }

    pub fn should_exit(&self) -> bool {
        self.should_exit
    }

    pub fn update(&mut self) -> Result<()> {
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
        frame.render_widget(
            Paragraph::new("Hier wird gebaut!").block(Block::new().borders(Borders::ALL)),
            layout[1],
        );
        frame.render_widget(
            Paragraph::new(self.metadata()).block(Block::new().borders(Borders::ALL)),
            layout[2],
        );
    }

    fn metadata(&self) -> String {
        format!("Foo")
    }
}
