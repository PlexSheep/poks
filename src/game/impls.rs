use std::{
    fmt::{Debug, Display},
    ops::{Deref, DerefMut},
};

use poker::Card;
use rand::{distr::StandardUniform, prelude::Distribution};

use crate::{
    CU,
    game::{Action, Cards, CardsDynamic, Phase, PlayerState, Winner, show_cards},
    len_to_const_arr,
};

struct Shortened;

impl CardsDynamic {
    pub const fn new() -> Self {
        Self { inner: Vec::new() }
    }

    pub fn try_static<const N: usize>(self) -> Option<Cards<N>> {
        if N != self.len() {
            return None;
        }
        len_to_const_arr(&self.inner).ok()
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
            70..100 => Action::Raise(CU!(10)),
            100 => Action::Raise(CU!(100)),
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

impl From<&[Card]> for CardsDynamic {
    fn from(value: &[Card]) -> Self {
        Self {
            inner: value.into(),
        }
    }
}

impl From<Vec<Card>> for CardsDynamic {
    fn from(value: Vec<Card>) -> Self {
        Self {
            inner: value.into(),
        }
    }
}
