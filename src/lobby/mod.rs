use circular_queue::CircularQueue;
use std::fmt::Debug;
use tracing::trace;

use crate::Result;
use crate::errors::PoksError;
use crate::game::{Game, PlayerID};
use crate::transaction::Transaction;

mod behavior;
mod seat;
pub use behavior::*;
pub use seat::*;

pub const ACTION_LOG_SIZE: usize = 2000;

#[derive(Debug)]
pub struct Lobby {
    players: Vec<Seat>,
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

    pub fn add_player(&mut self, player: BehaveBox) -> Result<&mut Self> {
        let seat: Seat = player.into();
        self.players.push(seat);

        Ok(self)
    }

    pub fn build(self) -> Result<Lobby> {
        trace!("Building Lobby");
        let mut w = Lobby {
            game: Game::build(&self.players, 0).unwrap(), // dummy
            players: self.players,
            action_log: CircularQueue::with_capacity(ACTION_LOG_SIZE),
            games_played: 0,
        };
        trace!("Starting first game");
        w.start_new_game()?;
        for player in &w.players {
            assert!(player.behavior().hand().is_some())
        }
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

        let dealer_pos = self.games_played as PlayerID % self.players.len();
        let game = Game::build(&self.players, dealer_pos)?;
        self.game = game;
        trace!("New game is ready");
        Ok(())
    }

    pub fn tick_game(&mut self) -> Result<()> {
        if self.game.is_finished() {
            return Err(PoksError::GameFinished);
        }
        debug_assert!(self.game.turn() < self.players.len());
        let pid = self.game.turn();
        let player = &mut self.players[pid];
        let action = player.behavior_mut().act(&self.game)?;
        let possible_transaction = action.map(|a| a.prepare_transaction());
        let res = match self.game.process_action(action) {
            Ok(_) => Ok(()),
            Err(e) => Err(e),
        };
        if let Some(Some(transaction)) = possible_transaction {
            // NOTE: adding is done by the game functionalities
            transaction.finish(player.behavior_mut().currency_mut(), Transaction::garbage())?;
        }
        self.update_action_log();
        res
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

    pub fn players(&self) -> &[Seat] {
        &self.players
    }
}
