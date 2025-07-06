use std::fmt::Debug;
use std::sync::OnceLock;

use poker::{Card, Eval, Evaluator, FiveCard};

use crate::currency::{self, Currency};
use crate::player::{PlayerBehavior, PlayerState};
use crate::transaction::Transaction;
use crate::{CU, Result, player_impl};

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

#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct Player {
    state: PlayerState,
    total_bet: Currency,
    round_bet: Currency,
    hand: Cards<2>,
}

#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct Game {
    phase: Phase,
    turn: PlayerID,
    players: Vec<Player>,
    community_cards: CardsDynamic,
    winner: Option<Winner>,
    deck: CardsDynamic,
    state: GameState,
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum Action {
    Fold,
    Call(Currency),
    Raise(Currency),
    AllIn(Currency),
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord, Default)]
#[non_exhaustive]
pub enum GameState {
    #[default]
    RaiseAllowed,
    RaiseDisallowed,
    Pause,
    Finished,
}

// helper macros
macro_rules! current_player {
    ($self:tt) => {
        $self.players[$self.turn]
    };
}

impl Game {
    pub fn build(player_amount: usize) -> Result<Self> {
        assert!(player_amount >= 2);
        let mut deck: CardsDynamic = poker::deck::shuffled().into();
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
            state: GameState::default(),
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
    pub fn highest_bet_of_round(&self) -> Currency {
        debug_assert!(!self.players.is_empty());
        self.players.iter().map(|p| p.round_bet).max().unwrap()
    }

    #[must_use]
    pub fn is_finished(&self) -> bool {
        self.winner.is_some()
    }

    pub fn set_winner(&mut self, w: Winner) {
        self.winner = Some(w);
    }

    #[must_use]
    pub fn winner(&self) -> Option<Winner> {
        self.winner
    }

    fn draw_card(&mut self) -> Card {
        self.deck.pop().unwrap()
    }

    #[inline]
    fn add_table_card(&mut self) {
        let c = self.draw_card();
        self.community_cards.push(c);
    }

    fn advance_phase(&mut self) {
        match self.phase() {
            Phase::Preflop => {
                let _ = self.draw_card(); // burn card
                for _ in 0..3 {
                    self.add_table_card();
                }
                assert_eq!(self.community_cards.len(), 3);
                self.set_phase(Phase::Flop);
            }
            Phase::Flop => {
                let _ = self.draw_card(); // burn card
                self.add_table_card();
                assert_eq!(self.community_cards.len(), 4);
                self.set_phase(Phase::Turn);
            }
            Phase::Turn => {
                let _ = self.draw_card(); // burn card
                self.add_table_card();
                assert_eq!(self.community_cards.len(), 5);
                self.set_phase(Phase::River);
                let w = self.showdown();
            }
            Phase::River => unreachable!(),
        }
    }

    fn showdown(&mut self) -> Result<Winner> {
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

    fn next_turn(&mut self) {
        self.turn = (self.turn + 1) % self.players.len();
        if self.turn == 0 {
            self.advance_phase();
        }
    }

    pub fn process_action(&mut self, action: Option<Action>) -> Result<()> {
        let remaining_players = self.players.iter().filter(|p| p.state.is_playing()).count();
        if remaining_players == 1 {
            todo!("Player is the last remaining one, let them win without showing cards")
        }

        let round_bet = self.highest_bet_of_round();
        let player = &current_player!(self);

        if !player.state.is_playing() {
            self.next_turn();
        }

        let action = match action {
            Some(a) => a,
            None => return Ok(()), // come back with an action
        };

        if !current_player!(self).state.is_playing() {
            todo!("Error: player not playing and cant make action")
        }

        if current_player!(self).state == PlayerState::AllIn {
            self.next_turn();
            return Ok(());
        }
        match action {
            Action::Fold => {
                current_player!(self).state = PlayerState::Folded;
            }
            Action::Call(currency) => {
                if round_bet <= current_player!(self).round_bet {
                    todo!("Cannot call when you are not under the round bet")
                }
                let diff = round_bet - current_player!(self).round_bet;
                if diff != currency {
                    todo!("Error: currency call mismatch ({diff} != {currency})")
                }
                if currency != CU!(0) {
                    current_player!(self).round_bet += currency;
                }
            }
            Action::Raise(currency) => {
                if self.state == GameState::RaiseDisallowed {
                    todo!("No betting allowed, just calling")
                }
                current_player!(self).round_bet += currency;
            }
            Action::AllIn(currency) => {
                if current_player!(self).state == PlayerState::AllIn {
                    todo!("Error: player is already all in")
                }
                if self.state != GameState::RaiseDisallowed {
                    todo!("No betting allowed, just calling")
                }
                current_player!(self).state = PlayerState::AllIn;
                current_player!(self).round_bet += currency;
            }
        }

        self.next_turn();
        Ok(())
    }

    pub fn show_table(&self) -> String {
        let mut buf = String::new();

        for i in 0..5 {
            let card: String = self
                .community_cards
                .get(i)
                .map(|c| c.to_string())
                .unwrap_or("[    ]".to_string());
            buf.push_str(&card);
        }

        buf
    }

    pub fn turn(&self) -> PlayerID {
        self.turn
    }

    pub fn players(&self) -> &[Player] {
        &self.players
    }

    pub fn community_cards(&self) -> &CardsDynamic {
        &self.community_cards
    }

    pub fn deck(&self) -> &CardsDynamic {
        &self.deck
    }

    pub fn state(&self) -> GameState {
        self.state
    }

    pub fn action_check(&self) -> Action {
        let diff = self.highest_bet_of_round() - self.players[self.turn].round_bet;
        Action::Call(diff)
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

    pub fn hand(&self) -> [Card; 2] {
        self.hand
    }

    pub fn state(&self) -> PlayerState {
        self.state
    }

    pub fn total_bet(&self) -> Currency {
        self.total_bet
    }

    pub fn round_bet(&self) -> Currency {
        self.round_bet
    }
}

impl GameState {
    #[inline]
    #[must_use]
    pub fn is_ongoing(&self) -> bool {
        match self {
            GameState::RaiseAllowed | GameState::RaiseDisallowed => true,
            GameState::Pause | GameState::Finished => false,
        }
    }
}

impl Action {
    pub fn prepare_transaction(&self) -> Option<Transaction> {
        match self {
            Action::Call(currency) | Action::Raise(currency) | Action::AllIn(currency) => {
                Some(Transaction::new(*currency))
            }
            Action::Fold => None,
        }
    }
    #[inline]
    pub fn check() -> Self {
        Self::Call(CU!(0))
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
