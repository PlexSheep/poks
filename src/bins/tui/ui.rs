use color_eyre::Result;
use crossterm::event::{Event, KeyCode, KeyModifiers};
use poks::{
    CU,
    currency::Currency,
    game::{Action, PlayerID},
    lobby::Lobby,
    player::PlayerCPU,
};
use tracing::{debug, info, trace};

use crate::player::local::{ActionAccessor, PlayerLocal};

mod render;

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord, Default)]
pub(crate) enum InputMode {
    #[default]
    Normal,
    Bet,
}

pub(crate) struct PoksTUI {
    world: Lobby,
    should_exit: bool,
    frame: u32,
    message: Option<String>,
    player_af: ActionAccessor,
    player_id: PlayerID,
    input_mode: InputMode,
    bet: Option<Currency>,
}

impl PoksTUI {
    pub(crate) fn new() -> Self {
        let mut worldb = Lobby::builder();

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
            bet: None,
            input_mode: Default::default(),
        }
    }

    pub(crate) fn should_exit(&self) -> bool {
        self.should_exit
    }

    pub(crate) fn update(&mut self) -> Result<()> {
        self.frame += 1;
        if self.world().game.is_finished() {
            self.message = Some("Game finished. Press F6 or Space for a new game.".to_string());
        } else {
            self.world.tick_game()?;
        }

        Ok(())
    }

    pub(crate) fn handle_event(&mut self, event: Event) -> Result<()> {
        trace!("Processing event {:?} with mode={}", event, self.input_mode);
        self.handle_input_base(event.clone())?;
        match self.input_mode {
            InputMode::Normal => self.handle_input_normal(event),
            InputMode::Bet => self.handle_input_bet(event),
        }
    }

    fn handle_input_base(&mut self, event: Event) -> Result<()> {
        if let Event::Key(key) = event {
            match key.code {
                KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                    info!("should exit");
                    self.should_exit = true
                }
                _ => (),
            }
        }
        Ok(())
    }

    fn handle_input_normal(&mut self, event: Event) -> Result<()> {
        if let Event::Key(key) = event {
            match key.code {
                KeyCode::Char('q') => {
                    info!("should exit");
                    self.should_exit = true
                }
                KeyCode::F(6) | KeyCode::Char(' ') | KeyCode::Enter
                    if self.world().game.is_finished() =>
                {
                    self.start_new_game()
                }
                KeyCode::F(1) => PlayerLocal::set_action(&self.player_af, Action::Fold),
                // TODO: call needs calculation of diff
                KeyCode::F(2) => {
                    PlayerLocal::set_action(&self.player_af, self.world().game.action_call())
                }
                KeyCode::F(3) => self.set_input_mode(InputMode::Bet),
                KeyCode::F(4) => PlayerLocal::set_action(
                    &self.player_af,
                    Action::AllIn(*self.world().players()[self.player_id].currency()),
                ),
                _ => (),
            }
        }
        Ok(())
    }

    fn set_input_mode(&mut self, mode: InputMode) {
        self.input_mode = mode;
        if mode == InputMode::Bet {
            self.bet = Some(self.world.game.big_blind())
        }
    }

    fn handle_input_bet(&mut self, event: Event) -> Result<()> {
        debug!("Input mode received key: {:?}", event);
        if let Event::Key(key) = event {
            match key.code {
                KeyCode::Esc => {
                    self.set_input_mode(InputMode::Normal);
                }
                KeyCode::Char('*') => {
                    let bet: &mut Currency = self.bet.get_or_insert_default();
                    *bet += self.world.game.big_blind() * 10;
                }
                KeyCode::Char('+') if key.modifiers.contains(KeyModifiers::ALT) => {
                    let bet: &mut Currency = self.bet.get_or_insert_default();
                    *bet += self.world.game.big_blind() * 100;
                }
                KeyCode::Char('+') => {
                    let bet: &mut Currency = self.bet.get_or_insert_default();
                    *bet += self.world.game.big_blind();
                }
                KeyCode::Char('_') => {
                    let bet: &mut Currency = self.bet.get_or_insert_default();
                    *bet -= self.world.game.big_blind() * 10;
                }
                KeyCode::Char('-') if key.modifiers.contains(KeyModifiers::ALT) => {
                    let bet: &mut Currency = self.bet.get_or_insert_default();
                    *bet -= self.world.game.big_blind() * 100;
                }
                KeyCode::Char('-') => {
                    let bet: &mut Currency = self.bet.get_or_insert_default();
                    *bet -= self.world.game.big_blind();
                }
                KeyCode::Enter => {
                    PlayerLocal::set_action(
                        &self.player_af,
                        Action::Raise(self.bet.take().unwrap()),
                    );
                    self.set_input_mode(InputMode::Normal);
                }
                _ => (),
            }
        }
        Ok(())
    }

    pub(crate) fn world(&self) -> &Lobby {
        &self.world
    }

    pub(crate) fn start_new_game(&mut self) {
        self.message = None;
        self.world
            .start_new_game()
            .expect("could not start new game");
    }
}
