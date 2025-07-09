use circular_queue::CircularQueue;
use std::fmt::Debug;
use tracing::{debug, trace};

use crate::Result;
use crate::errors::PoksError;
use crate::game::Game;
use crate::players::{PlayerID, Seat};

pub const ACTION_LOG_SIZE: usize = 2000;

#[derive(Debug)]
pub struct Lobby {
    seats: Vec<Seat>,
    pub game: Game,
    action_log: CircularQueue<(Option<PlayerID>, String)>,
    games_played: u64,
}

#[derive(Debug, Default)]
pub struct LobbyBuilder {
    pub players: Vec<Seat>,
}

impl LobbyBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_seat(&mut self, seat: Seat) -> Result<&mut Self> {
        self.players.push(seat);

        Ok(self)
    }

    pub fn build(self) -> Result<Lobby> {
        trace!("Building Lobby");
        let mut w = Lobby {
            game: Game::build(&self.players, 0).unwrap(), // dummy
            seats: self.players,
            action_log: CircularQueue::with_capacity(ACTION_LOG_SIZE),
            games_played: 0,
        };
        trace!("Starting first game");
        w.start_new_game()?;
        trace!("Lobby ready");
        Ok(w)
    }
}

impl Lobby {
    pub fn builder() -> LobbyBuilder {
        LobbyBuilder::default()
    }

    pub fn start_new_game(&mut self) -> Result<()> {
        trace!("Lobby starts a new game");
        self.games_played += 1;

        let dealer_pos = self.games_played as PlayerID % self.seats.len();
        let game = Game::build(&self.seats, dealer_pos)?;
        self.game = game;
        trace!("New game is ready");
        Ok(())
    }

    pub fn tick_game(&mut self) -> Result<()> {
        if self.game.is_finished() {
            return Err(PoksError::GameFinished);
        }
        debug_assert!(self.game.turn() < self.seats.len());
        let game = self.game.clone();
        let player = &mut self.game.current_player_mut();
        let action = player.act(&game)?;
        if let Some(action) = action {
            let res = match self.game.process_action(action) {
                Ok(_) => Ok(()),
                Err(e) => Err(e),
            };
            self.update_action_log();
            res
        } else {
            debug!("player.act did not return an action");
            Ok(())
        }
    }

    fn update_action_log(&mut self) {
        let glog = self.game.take_gamelog();
        for i in glog.into_iter() {
            self.action_log.push(i);
        }
    }

    pub fn action_log(&self) -> &CircularQueue<(Option<PlayerID>, String)> {
        &self.action_log
    }

    pub fn seats(&self) -> &[Seat] {
        &self.seats
    }
}
