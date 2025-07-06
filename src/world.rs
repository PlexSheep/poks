use circular_queue::CircularQueue;
use std::fmt::Debug;
use tracing::warn;

use crate::Result;
use crate::game::{Game, PlayerID};
use crate::player::PlayerBehavior;

pub const ACTION_LOG_SIZE: usize = 2000;

pub struct World {
    players: Vec<Box<dyn PlayerBehavior>>,
    pub game: Game,
    action_log: CircularQueue<(Option<PlayerID>, String)>,
}

#[derive(Debug, Default)]
pub struct WorldBuilder {
    players: Vec<Box<dyn PlayerBehavior>>,
}

impl WorldBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_player(&mut self, player: Box<dyn PlayerBehavior>) -> Result<&mut Self> {
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
            todo!("Game is finished, add error")
        }
        debug_assert!(self.game.turn() < self.players.len());
        let pid = self.game.turn();
        let player = &mut self.players[pid];
        let player_action = player.act(&self.game)?;
        if let Some(action) = player_action {
            match self.game.process_action(action) {
                Ok(_) => {
                    self.action_log.push((Some(pid), action.to_string()));
                    Ok(())
                }
                Err(e) => return Err(e),
            }
        } else {
            warn!(
                "Player {} has not made an action, waiting for them...",
                self.game.turn()
            );
            Ok(())
        }
    }

    pub fn action_log(&self) -> &CircularQueue<(Option<PlayerID>, String)> {
        &self.action_log
    }

    pub fn players(&self) -> &[Box<dyn PlayerBehavior>] {
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
