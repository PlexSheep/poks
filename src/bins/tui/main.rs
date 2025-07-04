use std::{fs::OpenOptions, time::Duration};

use color_eyre::Result;
use crossterm::event;
use ratatui::{DefaultTerminal, prelude::*};
use tracing::Level;
use tracing_subscriber::{Registry, filter, fmt, prelude::*};

use crate::ui::PoksTUI;

mod ui;

const EVENT_POLL_TIMEOUT: Duration = Duration::from_millis(1);

fn main() -> Result<()> {
    logging_setup();
    color_eyre::install()?;
    let terminal = ratatui::init();
    let result = run(terminal);
    ratatui::restore();
    result
}

fn logging_setup() {
    let logfile = OpenOptions::new()
        .append(true)
        .create(true)
        .open("poks.log")
        .unwrap();

    let subscriber = Registry::default().with(
        fmt::layer()
            .with_writer(logfile)
            .with_filter(filter::LevelFilter::from_level(Level::TRACE)),
    );

    tracing::subscriber::set_global_default(subscriber).unwrap();
}

fn run(mut terminal: DefaultTerminal) -> Result<()> {
    let mut tui = PoksTUI::new();

    while !tui.should_exit() {
        tui.update()?;
        terminal.draw(|f| tui.render(f))?;

        let event = event::read()?;
        tui.handle_event(event)?
    }

    Ok(())
}
