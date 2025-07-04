use std::{
    fmt::{Debug, Display},
    ops::{Index, IndexMut},
    usize,
};

use poker::Card;

use crate::game::{Hand, Phase, World};

struct Shortened;

impl Debug for Shortened {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "(..)")
    }
}

impl Debug for World {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("World")
            .field("evaluator", &Shortened)
            .field("players", &self.players)
            .field("game", &self.game)
            .finish()
    }
}

impl Display for Phase {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl From<(Card, Card)> for Hand {
    fn from(value: (Card, Card)) -> Self {
        Hand(value.0, value.1)
    }
}

impl From<[Card; 2]> for Hand {
    fn from(value: [Card; 2]) -> Self {
        Hand(value[0], value[1])
    }
}

impl Index<usize> for Hand {
    type Output = Card;

    fn index(&self, index: usize) -> &Self::Output {
        if index > 2 {
            panic!("Index too large: Only two cards per hand")
        }
        match index {
            0 => &self.0,
            1 => &self.1,
            _ => unreachable!(),
        }
    }
}

impl IndexMut<usize> for Hand {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        if index > 2 {
            panic!("Index too large: Only two cards per hand")
        }
        match index {
            0 => &mut self.0,
            1 => &mut self.1,
            _ => unreachable!(),
        }
    }
}

impl Display for Hand {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}{}", self.0, self.1)
    }
}
