use std::fmt::Debug;
use std::sync::Arc;

use poker::{Card, Eval, Evaluator, FiveCard};

mod impls; // additional trait impls

type Hand = [Card; 5];

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord, Default)]
pub enum Phase {
    #[default]
    Preflop,
    Flop,
    Turn,
    River,
}

#[derive(Clone, PartialEq, Eq)]
pub struct World {
    evaluator: Arc<Evaluator>,
    player_amount: usize,
    pub current_game: Option<Game>,
}

#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct Game {
    pub players: Vec<Player>,
    pub phase: Phase,
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum Player {
    Local(PlayerLocal),
    CPU(PlayerCPU),
}

pub trait PlayerBehavior {
    fn hand(&self) -> Option<&Hand>;
    fn hand_mut(&mut self) -> Option<&mut Hand>;
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord, Default)]
pub struct PlayerLocal {
    hand: Option<Hand>,
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord, Default)]
pub struct PlayerCPU {
    hand: Option<Hand>,
}

impl PlayerBehavior for Player {
    fn hand(&self) -> Option<&Hand> {
        match self {
            Player::Local(p) => p.hand(),
            Player::CPU(p) => p.hand(),
        }
    }

    fn hand_mut(&mut self) -> Option<&mut Hand> {
        match self {
            Player::Local(p) => p.hand_mut(),
            Player::CPU(p) => p.hand_mut(),
        }
    }
}

impl PlayerBehavior for PlayerLocal {
    fn hand(&self) -> Option<&Hand> {
        self.hand.as_ref()
    }

    fn hand_mut(&mut self) -> Option<&mut Hand> {
        self.hand.as_mut()
    }
}

impl PlayerBehavior for PlayerCPU {
    fn hand(&self) -> Option<&Hand> {
        self.hand.as_ref()
    }

    fn hand_mut(&mut self) -> Option<&mut Hand> {
        self.hand.as_mut()
    }
}

impl World {
    pub fn new(players: usize) -> Self {
        let evaluator = Evaluator::new().into();
        Self {
            evaluator,
            player_amount: players,
            current_game: None,
        }
    }

    pub fn start_new_game(&mut self) {
        self.current_game = Some(self.new_game())
    }

    pub fn new_game(&self) -> Game {
        let mut players = vec![Player::Local(Default::default())];
        for _ in 1..self.player_amount {
            players.push(Player::CPU(Default::default()))
        }
        Game {
            players,
            phase: Phase::default(),
        }
    }

    pub fn tick_game(&mut self) {
        todo!()
    }
}

impl Game {
    pub fn players(&self) -> &[Player] {
        &self.players
    }

    pub fn players_mut(&mut self) -> &mut Vec<Player> {
        &mut self.players
    }

    pub fn phase(&self) -> Phase {
        self.phase
    }

    pub fn phase_mut(&mut self) -> &mut Phase {
        &mut self.phase
    }

    pub fn set_phase(&mut self, phase: Phase) {
        self.phase = phase;
    }
}
