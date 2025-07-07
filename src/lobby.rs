use circular_queue::CircularQueue;
use std::fmt::Debug;

use crate::Result;
use crate::errors::PoksError;
use crate::game::{Game, PlayerID, Winner};
use crate::player::PlayerBehavior;
use crate::transaction::Transaction;

pub const ACTION_LOG_SIZE: usize = 2000;

pub type AnyAccount = Box<dyn PlayerBehavior>;

#[derive(Debug)]
pub struct Lobby {
    players: Vec<AnyAccount>,
    pub game: Game,
    action_log: CircularQueue<(Option<PlayerID>, String)>,
    games_played: u64,
}

#[derive(Debug, Default)]
pub struct LobbyBuilder {
    pub players: Vec<AnyAccount>,
}

impl LobbyBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_player(&mut self, player: AnyAccount) -> Result<&mut Self> {
        self.players.push(player);

        Ok(self)
    }

    pub fn build(mut self) -> Result<Lobby> {
        let mut w = Lobby {
            game: Game::build(self.players.len(), &mut self.players, 0).unwrap(), // dummy
            players: self.players,
            action_log: CircularQueue::with_capacity(ACTION_LOG_SIZE),
            games_played: 0,
        };
        w.start_new_game()?;
        for player in &w.players {
            assert!(player.hand().is_some())
        }
        Ok(w)
    }
}

impl Lobby {
    pub fn builder() -> LobbyBuilder {
        LobbyBuilder::default()
    }

    pub fn start_new_game(&mut self) -> Result<()> {
        self.games_played += 1;

        let dealer_pos = self.games_played as PlayerID % self.players.len();
        let game = Game::build(self.players.len(), &mut self.players, dealer_pos)?;
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
            Ok(_) => Ok(()),
            Err(e) => Err(e),
        };
        if let Some(Some(transaction)) = possible_transaction {
            // NOTE: adding is done by the game functionalities
            transaction.finish(player.currency_mut(), Transaction::garbage())?;
        }
        if self.game.is_finished() {
            let winner: Winner = self.game.winner().unwrap();
            let winning_player = &mut self.players[winner.pid()];
            winner.payout(&self.game, winning_player)?;
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

    pub fn players(&self) -> &[AnyAccount] {
        &self.players
    }
}
