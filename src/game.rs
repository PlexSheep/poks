use std::fmt::Debug;
use std::sync::OnceLock;

use poker::{Card, Eval, Evaluator, FiveCard};

use crate::Result;
use crate::currency::Currency;
use crate::player::PlayerState;

mod impls; // additional trait impls

pub type PlayerID = usize;
pub type Cards<const N: usize> = [Card; N];

pub static EVALUATOR: OnceLock<Evaluator> = OnceLock::new();

#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord, Default)]
pub struct CardsDynamic {
    inner: Vec<Card>,
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord, Default)]
pub enum Phase {
    #[default]
    Preflop,
    Flop,
    Turn,
    River,
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum Winner {
    UnknownCards(PlayerID),
    KnownCards(PlayerID, Eval<FiveCard>, Cards<7>),
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct Player {
    state: PlayerState,
    total_bet: Currency,
    round_bet: Currency,
    hand: Cards<2>,
}

#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct Game {
    pub phase: Phase,
    pub turn: PlayerID,
    pub players: Vec<Player>,
    pub community_cards: CardsDynamic,
    winner: Option<Winner>,
    deck: Vec<Card>,
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum Action {
    Fold,
    Call,
    Check,
    Raise(Currency),
    AllIn,
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
#[non_exhaustive]
pub enum GameState {
    Ongoing,
    Pause,
    Finished,
}

impl Game {
    pub fn build(player_amount: usize) -> Result<Self> {
        assert!(player_amount > 0);
        let mut deck = poker::deck::shuffled();
        if player_amount > deck.len() / 2 {
            // TODO: return a proper error and result
            panic!("Not enough cards in a deck for this many players!")
        }
        let mut players = Vec::new();
        for _ in 0..player_amount {
            let hand: Cards<2> = [deck.pop().unwrap(), deck.pop().unwrap()];
            players.push(Player::new(hand));
        }
        Ok(Game {
            turn: 0,
            phase: Phase::default(),
            players,
            community_cards: CardsDynamic::new(),
            winner: None,
            deck,
        })
    }

    #[must_use]
    pub fn phase(&self) -> Phase {
        self.phase
    }

    #[must_use]
    pub fn phase_mut(&mut self) -> &mut Phase {
        &mut self.phase
    }

    pub fn set_phase(&mut self, phase: Phase) {
        self.phase = phase;
    }

    #[must_use]
    pub fn pot(&self) -> Currency {
        debug_assert!(!self.players.is_empty());
        self.players.iter().map(|p| p.total_bet).sum()
    }

    #[must_use]
    pub fn highest_bet(&self) -> Currency {
        debug_assert!(!self.players.is_empty());
        self.players.iter().map(|p| p.total_bet).max().unwrap()
    }

    pub fn is_finished(&self) -> bool {
        self.winner.is_some()
    }

    pub fn set_winner(&mut self, w: Winner) {
        self.winner = Some(w);
    }

    pub fn winner(&self) -> Option<Winner> {
        self.winner
    }

    pub fn add_table_card(&mut self, card: Card) {
        self.community_cards.push(card);
    }

    pub fn advance_phase<F: FnOnce() -> Card>(&mut self, draw_card: F) {
        match self.phase() {
            Phase::Preflop => {
                let _ = draw_card(); // burn card
                for _ in 0..3 {
                    self.add_table_card(draw_card());
                }
                assert_eq!(self.community_cards.len(), 3);
                self.set_phase(Phase::Flop);
            }
            Phase::Flop => {
                let _ = draw_card(); // burn card
                self.add_table_card(draw_card());
                assert_eq!(self.community_cards.len(), 4);
                self.set_phase(Phase::Turn);
            }
            Phase::Turn => {
                let _ = draw_card(); // burn card
                self.add_table_card(draw_card());
                assert_eq!(self.community_cards.len(), 5);
                self.set_phase(Phase::River);
                let w = self.showdown()?;
                self.action_log.push((None, Action::Winner(w)));
            }
            Phase::River => unreachable!(),
        }
    }

    pub fn showdown(&mut self) -> Result<Winner> {
        let mut evals: Vec<(PlayerID, Eval<FiveCard>, Cards<7>)> = Vec::new();
        for (pid, player) in self.players.iter().enumerate() {
            if player.state != PlayerState::Playing {
                continue;
            }
            let mut hand_plus_table: CardsDynamic = player.hand.into();
            hand_plus_table.extend(self.community_cards.iter());
            // TODO: add better result type and return this as error
            evals.push((
                pid,
                evaluator()
                    .evaluate_five(&*hand_plus_table)
                    .expect("could not evaluate"),
                hand_plus_table
                    .try_static()
                    .expect("Hands plus table were not 7 cards"),
            ));
        }

        evals.sort_by(|a, b| a.1.cmp(&b.1));
        let winner = Winner::KnownCards(evals[0].0, evals[0].1, evals[0].2);
        self.set_winner(winner);

        Ok(winner)
    }

    #[must_use]
    pub fn process_action(&self, action: Action) -> Result<GameState> {
        todo!()
    }
}

impl Player {
    #[must_use]
    #[inline]
    pub fn show_hand(&self) -> String {
        show_cards(&self.hand)
    }

    pub fn new(hand: Cards<2>) -> Self {
        Self {
            state: Default::default(),
            total_bet: Default::default(),
            round_bet: Default::default(),
            hand,
        }
    }
}

pub fn show_cards(cards: &[Card]) -> String {
    let mut buf = String::new();
    for card in cards {
        buf.push_str(&card.to_string());
    }
    buf
}

#[inline]
pub fn evaluator() -> &'static Evaluator {
    EVALUATOR.get_or_init(|| Evaluator::new())
}
