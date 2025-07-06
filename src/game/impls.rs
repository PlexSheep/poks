use std::{
    fmt::{Debug, Display},
    ops::{Deref, DerefMut},
};

use poker::Card;
use rand::{distr::StandardUniform, prelude::Distribution};

use crate::game::{Action, Cards, CardsDynamic, Phase, PlayerState, Winner, show_cards};

struct Shortened;

impl CardsDynamic {
    pub const fn new() -> Self {
        Self { inner: Vec::new() }
    }
}

impl Debug for Shortened {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "(..)")
    }
}

impl Display for Phase {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}

impl Display for PlayerState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}

impl Display for Action {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Action::Fold => "folds".to_string(),
                Action::Call => "calls".to_string(),
                Action::Check => "checks".to_string(),
                Action::Raise(bet) => format!("raises by {bet}"),
                Action::AllIn => "goes all in!".to_string(),
            }
        )
    }
}

impl Distribution<Action> for StandardUniform {
    fn sample<R: rand::Rng + ?Sized>(&self, rng: &mut R) -> Action {
        let disc: u8 = rng.random_range(0..=70);
        match disc {
            0 => Action::Fold,
            1..70 => Action::Check,
            70..100 => Action::Raise(10),
            100 => Action::Raise(100),
            _ => unreachable!(),
        }
    }
}

impl Display for Winner {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::KnownCards(pid, eval, cards) => {
                    format!("Player {pid} won with {eval} ({}).", show_cards(cards))
                }
                Self::UnknownCards(pid) => format!("Player {pid} won."),
            }
        )
    }
}

impl<const N: usize> From<Cards<N>> for CardsDynamic {
    fn from(value: Cards<N>) -> Self {
        Self {
            inner: value.into(),
        }
    }
}

impl Deref for CardsDynamic {
    type Target = Vec<Card>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl DerefMut for CardsDynamic {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}
