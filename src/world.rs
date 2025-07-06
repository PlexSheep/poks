use circular_queue::CircularQueue;
use std::fmt::Debug;

use crate::Result;
use crate::game::{Game, GameState, PlayerID};
use crate::player::PlayerBehavior;

pub const ACTION_LOG_SIZE: usize = 2000;

pub struct World {
    players: Vec<Box<dyn PlayerBehavior>>,
    pub game: Game,
    action_log: CircularQueue<(Option<PlayerID>, String)>,
}

#[derive(Debug, Default)]
pub struct WorldBuilder {
    pub players: Vec<Box<dyn PlayerBehavior>>,
}

impl WorldBuilder {
    pub fn new() -> Self {
        WorldBuilder {
            players: Vec::with_capacity(4),
        }
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
    pub fn start_new_game(&mut self) -> Result<()> {
        let game = Game::build(self.players.len())?;
        self.game = game;
        Ok(())
    }

    pub fn tick_game(&mut self) -> Result<GameState> {
        if self.game.is_finished() {
            return Ok(GameState::Finished);
        }
        debug_assert!(self.game.turn < self.players.len());
        let player_action = self.players[self.game.turn].act(&self.game);
        self.game.process_action(player_action)
    }

    pub fn action_log(&self) -> &CircularQueue<(Option<PlayerID>, String)> {
        &self.action_log
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
