use std::sync::OnceLock;

pub use poker::{Eval, Evaluator, FiveCard, evaluate::FiveCardHandClass};

use crate::game::cards::{Card, Cards, Rank, Suit, show_cards};

pub static EVALUATOR: OnceLock<Evaluator> = OnceLock::new();

#[inline]
pub fn evaluator() -> &'static Evaluator {
    EVALUATOR.get_or_init(Evaluator::new)
}

pub fn show_eval_cards(cls: FiveCardHandClass, cards: &Cards<7>) -> String {
    assert!(cards.is_sorted());

    // HACK: These macros can likely be implemented with functions
    macro_rules! scards {
        ($collection:expr) => {{
            $collection.sort();
            $collection.reverse();
            $collection.truncate(5);
            debug_assert!($collection.len() <= 5); // BUG: this sometimes fails
            debug_assert!($collection.len() >= 1);
            $collection
        }};
    }
    macro_rules! filter {
        ($cards:tt, $filter:expr) => {{
            let mut _v: Vec<&Card> = $cards.into_iter().rev().filter($filter).collect();
            _v
        }};
    }
    macro_rules! fcards {
        ($filter:expr) => {{
            let mut _filter = filter!(cards, $filter);
            scards!(_filter)
        }};
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
            debug_assert_eq!(longest.len(), 5);
            longest.clone()
        }};
    }
    // PERF: This can likely be implemented more efficiently
    macro_rules! straight {
        ($cards:tt, $rank:tt) => {{
            let mut v: Vec<&Card> = Vec::with_capacity(5);
            let mut ranks = [
                Rank::Two,
                Rank::Three,
                Rank::Four,
                Rank::Five,
                Rank::Six,
                Rank::Seven,
                Rank::Eight,
                Rank::Nine,
                Rank::Ten,
                Rank::Jack,
                Rank::Queen,
                Rank::King,
                Rank::Ace,
            ];
            ranks.reverse();
            let mut nr: usize = ranks.iter().position(|r| *r == $rank).unwrap();
            let mut next_rank = $rank;
            for _ in 0..5 {
                v.push(
                    cards
                        .iter()
                        .filter(|c| c.rank() == next_rank)
                        .collect::<Vec<_>>()[0],
                );
                nr = (nr + 1) % ranks.len();
                next_rank = ranks[nr];
            }
            v.truncate(5);
            debug_assert!(v.len() <= 5);
            v.sort();
            v.reverse();
            v
        }};
    }
    let cards: Vec<&Card> = match cls {
        FiveCardHandClass::HighCard { .. } => vec![&cards[6]],
        FiveCardHandClass::Pair { rank } => fcards!(|c| c.rank() == rank),
        FiveCardHandClass::TwoPair {
            high_rank,
            low_rank,
        } => fcards!(|c| c.rank() == high_rank || c.rank() == low_rank),
        FiveCardHandClass::ThreeOfAKind { rank } => fcards!(|c| c.rank() == rank),
        FiveCardHandClass::Straight { rank } => {
            scards!(straight!(cards, rank))
        }
        FiveCardHandClass::Flush { .. } => scards!(flush!(cards)),
        FiveCardHandClass::FullHouse { trips, pair } => {
            // BUG: sometimes, an assert here fails
            fcards!(|c| c.rank() == pair || c.rank() == trips)
        }
        FiveCardHandClass::FourOfAKind { rank } => fcards!(|c| c.rank() == rank),
        #[allow(unused_variables)] // false positive
        FiveCardHandClass::StraightFlush { rank } => {
            let f: Vec<&Card> = flush!(cards);
            let mut s: Vec<&Card> = straight!(f, rank);
            scards!(s)
        }
    };
    show_cards(&cards)
}

#[cfg(test)]
mod test {

    use poker::{Card, cards};

    use super::*;
    use crate::utils::len_to_const_arr;

    #[test]
    fn test_show_eval_cards() {
        let r: Vec<(Vec<_>, &str)> = vec![
            (cards!("Th 2c 3c 4c 5c 7h 8h").collect(), "[ T♥ ]"), // high card
            (cards!("Th Tc 3c 4c 5c 7h 8h").collect(), "[ T♥ ][ T♣ ]"), // pair
            (
                cards!("Th Tc 3c 3h 5c 7h 8h").collect(),
                "[ T♥ ][ T♣ ][ 3♣ ][ 3♥ ]",
            ), // two pair
            (
                cards!("Th Tc Td 5c 6h 7h 8h").collect(),
                "[ T♥ ][ T♣ ][ T♦ ]",
            ), // set
            (
                cards!("Th 3c 4c 5c 6h 7h 8h").collect(),
                "[ 8♥ ][ 7♥ ][ 6♥ ][ 5♣ ][ 4♣ ]",
            ), // straight
            (
                cards!("Ah 3c 4c 2c 5h 7h 8h").collect(),
                "[ A♥ ][ 5♥ ][ 4♣ ][ 3♣ ][ 2♣ ]",
            ), // straight that wraps around
            (
                cards!("Th 3h 4h 5c 6h 7h 8h").collect(),
                "[ T♥ ][ 8♥ ][ 7♥ ][ 6♥ ][ 4♥ ]",
            ), // flush
            (
                cards!("Th Tc Td 5c 5h 7h 8h").collect(),
                "[ T♥ ][ T♣ ][ T♦ ][ 5♣ ][ 5♥ ]",
            ), // full house
            (
                cards!("Th Tc Td Ts 6h 7h 8h").collect(),
                "[ T♥ ][ T♣ ][ T♦ ][ T♠ ]",
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
