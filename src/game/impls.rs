use std::{
    fmt::Display,
    ops::{Deref, DerefMut},
};

use poker::{Card, Rank, Suit, cards, evaluate::FiveCardHandClass};
use rand::{distr::StandardUniform, prelude::Distribution};

use crate::{
    CU,
    game::{Action, Cards, CardsDynamic, Phase, PlayerState, Winner, show_cards},
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

impl Distribution<Action> for StandardUniform {
    fn sample<R: rand::Rng + ?Sized>(&self, rng: &mut R) -> Action {
        let disc: u8 = rng.random_range(0..=70);
        match disc {
            0 => Action::Fold,
            1..70 => Action::check(),
            70..100 => Action::Raise(CU!(10)),
            100 => Action::Raise(CU!(100)),
            _ => unreachable!(),
        }
    }
}

fn show_eval_cards(cls: FiveCardHandClass, cards: &Cards<7>) -> String {
    assert!(cards.is_sorted());

    let todo = String::from("todo");
    macro_rules! scards {
        ($collection:expr) => {{
            $collection.sort();
            $collection.reverse();
            $collection.into_iter().map(|c| c.to_string()).collect()
        }};
    }
    macro_rules! filter {
        ($cards:tt, $filter:expr) => {{
            let mut _v: Vec<_> = $cards.iter().rev().filter($filter).collect();
            _v
        }};
    }
    macro_rules! fcards {
        ($filter:expr) => {
            scards!(filter!(cards, $filter))
        };
    }
    macro_rules! flush {
        ($cards:tt) => {{
            let mut v: [Vec<&Card>; 4] = [
                filter!($cards, |c| c.suit() == Suit::Clubs),
                filter!($cards, |c| c.suit() == Suit::Hearts),
                filter!($cards, |c| c.suit() == Suit::Spades),
                filter!($cards, |c| c.suit() == Suit::Diamonds),
            ];
            v.sort_by_key(|b| std::cmp::Reverse(b.len()));
            let longest = &mut v[0];
            longest.truncate(5);
            longest.clone()
        }};
    }
    match cls {
        FiveCardHandClass::HighCard { .. } => cards[6].to_string(),
        FiveCardHandClass::Pair { rank } => fcards!(|c| c.rank() == rank),
        FiveCardHandClass::TwoPair {
            high_rank,
            low_rank,
        } => fcards!(|c| c.rank() == high_rank || c.rank() == low_rank),
        FiveCardHandClass::ThreeOfAKind { rank } => fcards!(|c| c.rank() == rank),
        FiveCardHandClass::Straight { rank } => {
            let mut v = filter!(cards, |c| c.rank() <= rank);
            v.truncate(5);
            v.reverse();
            scards!(v)
        }
        FiveCardHandClass::Flush { .. } => scards!(flush!(cards)),
        FiveCardHandClass::FullHouse { trips, pair } => {
            fcards!(|c| c.rank() == pair || c.rank() == trips)
        }
        FiveCardHandClass::FourOfAKind { rank } => fcards!(|c| c.rank() == rank),
        FiveCardHandClass::StraightFlush { rank } => {
            let flushcards: Vec<&Card> = flush!(cards);
            let mut v = filter!(flushcards, |c| c.rank() <= rank);
            v.truncate(5);
            v.reverse();
            scards!(v)
        }
    }
    // show_cards(&cards)
}

impl Display for Winner {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::KnownCards(pot, pid, eval, cards) => {
                    format!(
                        "Player {pid} won {pot} with {eval}:\n{}.",
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

#[cfg(test)]
mod test {
    use poker::{Card, cards};

    use crate::{
        game::{evaluator, impls::show_eval_cards},
        len_to_const_arr,
    };

    #[test]
    fn test_show_eval_cards() {
        let r: Vec<(Vec<_>, &str)> = vec![
            (cards!("Th 2c 3c 4c 5c 7h 8h").collect(), "[ T♥ ]"), // high card
            (cards!("Th Tc 3c 4c 5c 7h 8h").collect(), "[ T♣ ][ T♥ ]"), // pair
            (
                cards!("Th Tc 3c 3h 5c 7h 8h").collect(),
                "[ T♣ ][ T♥ ][ 3♥ ][ 3♣ ]",
            ), // two pair
            (
                cards!("Th Tc Td 5c 6h 7h 8h").collect(),
                "[ T♦ ][ T♣ ][ T♥ ]",
            ), // set
            (
                cards!("Th 3c 4c 5c 6h 7h 8h").collect(),
                "[ 8♥ ][ 7♥ ][ 6♥ ][ 5♣ ][ 4♣ ]",
            ), // straight
            (
                cards!("Th 3h 4h 5c 6h 7h 8h").collect(),
                "[ T♥ ][ 8♥ ][ 7♥ ][ 6♥ ][ 4♥ ]",
            ), // flush
            (
                cards!("Th Tc Td 5c 5h 7h 8h").collect(),
                "[ T♦ ][ T♣ ][ T♥ ][ 5♥ ][ 5♣ ]",
            ), // full house
            (
                cards!("Th Tc Td Ts 6h 7h 8h").collect(),
                "[ T♠ ][ T♦ ][ T♣ ][ T♥ ]",
            ), // quads
            (
                cards!("9h 3c 4h 5h 6h 7h 8h").collect(),
                "[ 9♥ ][ 8♥ ][ 7♥ ][ 6♥ ][ 5♥ ]",
            ), // straight flush
        ];
        for (cards, show) in r {
            let mut cards: Vec<Card> = cards.into_iter().map(|a| a.unwrap()).collect();
            cards.sort();
            let cards = len_to_const_arr(&cards).unwrap();
            assert_eq!(
                show_eval_cards(evaluator().evaluate_five(cards).unwrap().classify(), &cards),
                show
            );
        }
    }
}
