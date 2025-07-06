use circular_queue::CircularQueue;
use poker::{Card, Evaluator};
use std::sync::Arc;
use tracing::{debug, info};

use crate::game::{Action, Game, GameState, Hand, Phase, PlayerID};
use crate::player::{Player, PlayerBehavior, PlayerState};
use crate::{Result, len_to_const_arr};

mod impls; // trait impls

pub const ACTION_LOG_SIZE: usize = 2000;

pub struct World {
    evaluator: Arc<Evaluator>,
    players: Vec<Box<dyn PlayerBehavior>>,
    pub game: Game,
    deck: Vec<Card>,
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

    pub fn build(self) -> World {
        let evaluator = Evaluator::new().into();
        let deck = poker::deck::shuffled();
        debug_assert_eq!(deck.len(), 52);
        let mut w = World {
            evaluator,
            game: Game::new(self.players.len()), // dummy
            players: self.players,
            deck,
            action_log: CircularQueue::with_capacity(ACTION_LOG_SIZE),
        };
        w.start_new_game();
        for player in &w.players {
            assert!(player.hand().is_some())
        }
        w
    }
}

impl World {
    pub fn shuffle_cards(&mut self) {
        self.deck = poker::deck::shuffled();
        debug_assert_eq!(self.deck.len(), 52)
    }

    #[must_use]
    pub fn draw_card(&mut self) -> Card {
        self.deck.pop().expect("the deck was empty!")
    }

    pub fn start_new_game(&mut self) {
        self.shuffle_cards();
        let game = Game::new(self.players.len());

        for pi in 0..self.players.len() {
            let hand: Hand = [self.draw_card(), self.draw_card()].into();
            let player = &mut self.players[pi];
            player.set_hand(hand);
        }

        self.game = game;
    }

    pub fn tick_game(&mut self) -> Result<GameState> {
        if self.game.is_finished() {
            return Ok(GameState::Finished);
        }
        debug_assert!(self.game.turn < self.players.len());
        let player_action = self.players[self.game.turn].act(&self.game);
        self.game.process_action(player_action)
    }

    fn next_turn(&mut self) {
        self.game.turn = (self.game.turn + 1) % self.players.len();
        todo!("Advance phase if all players are done with this turn")
    }

    pub fn show_table(&self) -> String {
        let mut buf = String::new();

        for i in 0..5 {
            let card: String = self
                .game
                .community_cards
                .get(i)
                .map(|c| c.to_string())
                .unwrap_or("[    ]".to_string());
            buf.push_str(&card);
        }

        buf
    }

    pub fn action_log(&self) -> &CircularQueue<(Option<PlayerID>, String)> {
        &self.action_log
    }
}
