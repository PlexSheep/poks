mod state;
pub use state::*;

mod behavior;
pub use behavior::*;

pub mod cpu;
pub mod local;
pub use cpu::PlayerCPU;
pub use local::PlayerLocal;

mod seat;
pub use seat::*;

use std::fmt::Debug;

use crate::{
    Result,
    currency::Currency,
    game::{
        Action,
        cards::{Card, Cards, show_cards},
    },
};

pub type PlayerID = usize;
pub type BehaveBox = Box<dyn PlayerBehavior + Send + Sync>;

#[derive(Debug, Clone, PartialEq, Eq)]
#[must_use]
pub struct Player {
    pub state: PlayerState,
    pub total_bet: Currency,
    pub round_bet: Currency,
    pub hand: Cards<2>,
    pub seat: Seat,
}

#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord, Default)]
pub struct PlayerBasicFields {
    pub hand: Option<Cards<2>>,
    pub currency: Currency,
}

impl Player {
    #[must_use]
    #[inline]
    pub fn show_hand(&self) -> String {
        show_cards(&self.hand())
    }

    pub fn new(hand: Cards<2>, lobby_seat: Seat) -> Self {
        Self {
            state: Default::default(),
            total_bet: Default::default(),
            round_bet: Default::default(),
            seat: lobby_seat,
            hand,
        }
    }

    #[inline]
    pub fn set_hand(&mut self, hand: Cards<2>) {
        self.hand = hand;
    }

    #[inline]
    pub fn hand(&self) -> [Card; 2] {
        self.hand
    }

    #[inline]
    pub fn hand_mut(&mut self) -> &mut [Card; 2] {
        &mut self.hand
    }

    #[inline]
    pub fn state(&self) -> PlayerState {
        self.state
    }

    #[inline]
    pub fn total_bet(&self) -> Currency {
        self.total_bet + self.round_bet
    }

    #[inline]
    pub fn round_bet(&self) -> Currency {
        self.round_bet
    }

    #[inline]
    pub fn currency(&self) -> Currency {
        self.seat.currency()
    }

    #[inline]
    pub fn add_currency(&mut self, cu: Currency) -> Result<()> {
        self.seat.add_currency(cu)
    }

    #[inline]
    pub fn withdraw_currency(&mut self, cu: Currency) -> Result<Currency> {
        self.seat.withdraw_currency(cu)
    }

    #[inline]
    pub fn set_currency(&mut self, cu: Currency) {
        self.seat.set_currency(cu);
    }

    pub fn set_state(&mut self, state: PlayerState) {
        self.state = state;
    }

    pub fn state_mut(&mut self) -> &mut PlayerState {
        &mut self.state
    }

    pub fn total_bet_mut(&mut self) -> &mut Currency {
        &mut self.total_bet
    }

    pub fn set_total_bet(&mut self, total_bet: Currency) {
        self.total_bet = total_bet;
    }

    pub fn round_bet_mut(&mut self) -> &mut Currency {
        &mut self.round_bet
    }

    pub fn set_round_bet(&mut self, round_bet: Currency) {
        self.round_bet = round_bet;
    }

    pub fn seat(&self) -> &Seat {
        &self.seat
    }

    pub fn seat_mut(&mut self) -> &mut Seat {
        &mut self.seat
    }

    pub fn act(&self, game: &crate::game::Game) -> Result<Option<Action>> {
        self.seat.act(game, self)
    }

    pub fn is_active(&self) -> bool {
        self.state.is_playing()
    }
}
