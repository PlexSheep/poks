use std::{fmt::Display, fs::OpenOptions, time::Duration};

use clap::Parser;
use color_eyre::Result;
use crossterm::event;
use ratatui::DefaultTerminal;
use tracing::{Level, debug};
use tracing_subscriber::{Registry, filter, fmt, prelude::*};

use crate::ui::PoksTUI;

mod ui;

const EVENT_POLL_TIMEOUT: Duration = Duration::from_millis(1);
const HELP_TEMPLATE: &str = r"{about-section}
{usage-heading} {usage}

{all-args}{tab}

{name}: {version}
Author: {author-with-newline}
";

/// TUI texas holdem poker game using poks/poksen
#[derive(Parser)]
#[command(
    author = env!("CARGO_PKG_AUTHORS"),
    version = env!("CARGO_PKG_VERSION"),
    about = "Simple local backups with a bit of compression",
    help_template = HELP_TEMPLATE
)]
struct Cli {
    /// More verbose logs (to ./poks.log)
    #[arg(short, long)]
    verbose: bool,
    /// Total amount of players in the lobby
    #[arg(short, long, value_name = "PLAYERS")]
    players: u8,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    logging_setup(cli.verbose);
    color_eyre::install()?;
    let terminal = ratatui::init();
    let result = run(terminal, cli.players);
    ratatui::restore();
    result
}

fn logging_setup(verbose: bool) {
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
            .with_filter(filter::LevelFilter::from_level(if verbose {
                Level::TRACE
            } else {
                Level::DEBUG
            })),
    );
    #[cfg(not(debug_assertions))]
    let subscriber = Registry::default().with(fmt::layer().with_writer(logfile).with_filter(
        filter::LevelFilter::from_level(if verbose { Level::DEBUG } else { Level::INFO }),
    ));

    tracing::subscriber::set_global_default(subscriber).unwrap();
    debug!("Logging setup finished")
}

fn run(mut terminal: DefaultTerminal, players: u8) -> Result<()> {
    let mut tui = PoksTUI::new(players);

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
