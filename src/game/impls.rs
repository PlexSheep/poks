use std::{
    fmt::Display,
    ops::{Deref, DerefMut},
};

use poker::Card;
use rand::{distributions::Standard, prelude::Distribution};

use crate::{
    CU,
    game::{Action, Cards, CardsDynamic, Phase, PlayerState, Winner, show_eval_cards},
    len_to_const_arr,
};

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
                Action::Call(bet) if *bet == CU!(0) => "checks".to_string(),
                Action::Call(bet) => format!("calls for {bet}"),
                Action::Raise(bet) => format!("raises by {bet}"),
                Action::AllIn(bet) => format!("goes all in! ({bet})"),
            }
        )
    }
}

impl Distribution<Action> for Standard {
    fn sample<R: rand::Rng + ?Sized>(&self, rng: &mut R) -> Action {
        let disc: u8 = rng.gen_range(0..=70);
        match disc {
            0 => Action::Fold,
            1..70 => Action::check(),
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
                Self::KnownCards(pot, pid, eval, cards) => {
                    format!(
                        "Player {pid} won {pot} with {eval}:\n  {}",
                        show_eval_cards(eval.classify(), cards)
                    )
                }
                Self::UnknownCards(pot, pid) => format!("Player {pid} won {pot}."),
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
        Self { inner: value }
    }
}
