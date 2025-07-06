use circular_queue::CircularQueue;
use std::fmt::Debug;
use tracing::{trace, warn};

use crate::Result;
use crate::errors::PoksError;
use crate::game::{Game, PlayerID};
use crate::player::PlayerBehavior;
use crate::transaction::Transaction;

pub const ACTION_LOG_SIZE: usize = 2000;

pub type AnyPlayer = Box<dyn PlayerBehavior>;

pub struct World {
    players: Vec<AnyPlayer>,
    pub game: Game,
    action_log: CircularQueue<(Option<PlayerID>, String)>,
}

#[derive(Debug, Default)]
pub struct WorldBuilder {
    pub players: Vec<AnyPlayer>,
}

impl WorldBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_player(&mut self, player: AnyPlayer) -> Result<&mut Self> {
        self.players.push(player);

        Ok(self)
    }

    pub fn build(self) -> Result<World> {
        let mut w = World {
            game: Game::build(self.players.len()).unwrap(), // dummy
            players: self.players,
            action_log: CircularQueue::with_capacity(ACTION_LOG_SIZE),
        };
        w.start_new_game()?;
        for player in &w.players {
            assert!(player.hand().is_some())
        }
        Ok(w)
    }
}

impl World {
    pub fn builder() -> WorldBuilder {
        WorldBuilder::default()
    }

    pub fn start_new_game(&mut self) -> Result<()> {
        let game = Game::build(self.players.len())?;
        self.game = game;
        let players_game = self.game.players();
        assert_eq!(self.players.len(), players_game.len());
        for (gp, wp) in self.players.iter_mut().zip(players_game.iter()) {
            gp.set_hand(wp.hand());
        }
        Ok(())
    }

    pub fn tick_game(&mut self) -> Result<()> {
        if self.game.is_finished() {
            return Err(PoksError::GameFinished);
        }
        debug_assert!(self.game.turn() < self.players.len());
        let pid = self.game.turn();
        let player = &mut self.players[pid];
        let action = player.act(&self.game)?;
        let possible_transaction = action.map(|a| a.prepare_transaction());
        let res = match self.game.process_action(action) {
            Ok(_) if action.is_none() => Ok(()),
            Ok(_) => {
                self.action_log
                    .push((Some(pid), action.unwrap().to_string()));
                Ok(())
            }
            Err(e) => Err(e),
        };
        if let Some(Some(transaction)) = possible_transaction {
            // NOTE: adding is done by the game functionalities
            transaction.finish(player.currency_mut(), Transaction::garbage())?;
        }
        if self.game.is_finished() {
            self.action_log
                .push((None, self.game.winner().unwrap().to_string()));
        }
        res
    }

    pub fn action_log(&self) -> &CircularQueue<(Option<PlayerID>, String)> {
        &self.action_log
    }

    pub fn players(&self) -> &[AnyPlayer] {
        &self.players
    }
}

impl Debug for World {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("World")
            .field("players", &self.players)
            .field("game", &self.game)
            .field("action_log", &self.action_log)
            .finish()
    }
}
