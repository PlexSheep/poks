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
}

#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct Game {
    players: Vec<Player>,
    phase: Phase,
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum Player {
    Local(PlayerLocal),
    CPU(PlayerCPU),
}

pub trait PlayerBehavior: Default {
    fn hand(&self) -> Hand;
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord, Default)]
pub struct PlayerLocal {
    hand: Option<Hand>,
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord, Default)]
pub struct PlayerCPU {
    hand: Option<Hand>,
}

impl World {
    pub fn new(players: usize) -> Self {
        let evaluator = Evaluator::new().into();
        Self {
            evaluator,
            player_amount: players,
        }
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
