use std::{fs::OpenOptions, time::Duration};

use color_eyre::Result;
use crossterm::event;
use ratatui::DefaultTerminal;
use tracing::{Level, debug};
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

    #[cfg(debug_assertions)]
    let subscriber = Registry::default().with(
        fmt::layer()
            .with_writer(logfile)
            .with_target(true)
            .with_filter(filter::LevelFilter::from_level(Level::TRACE)),
    );
    #[cfg(not(debug_assertions))]
    let subscriber = Registry::default().with(
        fmt::layer()
            .with_writer(logfile)
            .with_filter(filter::LevelFilter::from_level(Level::TRACE)),
    );

    tracing::subscriber::set_global_default(subscriber).unwrap();
    debug!("Logging setup finished")
}

fn run(mut terminal: DefaultTerminal) -> Result<()> {
    let mut tui = PoksTUI::new();

    debug!("Starting the main loop");

    while !tui.should_exit() {
        terminal.draw(|f| tui.render(f))?;

        if event::poll(EVENT_POLL_TIMEOUT)? {
            let event = event::read()?;
            tui.handle_event(event)?;
        }
        tui.update()?;
        std::thread::sleep(Duration::from_millis(15));
    }

    Ok(())
}
