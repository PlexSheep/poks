use std::{
    fmt::Display,
    ops::{Deref, DerefMut},
};

pub use poker::{Card, Rank, Suit};

use crate::utils::len_to_const_arr;

pub type Cards<const N: usize> = [Card; N];

#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord, Default)]
pub struct CardsDynamic {
    inner: Vec<Card>,
}

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

pub fn show_cards(cards: &[impl Display]) -> String {
    let mut buf = String::new();
    for card in cards.iter() {
        buf.push_str(&card.to_string());
    }
    buf
}
