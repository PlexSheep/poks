mod state;
pub use state::*;

mod behavior;
pub use behavior::*;

pub mod cpu;
pub mod local;
pub use cpu::PlayerCPU;
pub use local::PlayerLocal;

use std::fmt::Debug;

use crate::currency::Currency;
use crate::game::cards::{Card, Cards, show_cards};
use crate::lobby::Seat;

pub type PlayerID = usize;

#[derive(Debug, Clone)]
pub struct Player {
    pub state: PlayerState,
    pub total_bet: Currency,
    pub round_bet: Currency,
    pub seat: Seat,
}

#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord, Default)]
pub struct PlayerBasicFields {
    pub hand: Option<Cards<2>>,
    pub currency: Currency,
}

#[macro_export]
macro_rules! player_impl {
    ($struct:ident, $base_field:tt, $($extra:tt)+) => {
        impl $crate::players::PlayerBehavior for $struct {
            fn hand(&self) -> &Option<$crate::game::cards::Cards<2>> {
                &self.$base_field.hand
            }

            fn hand_mut(&mut self) -> &mut Option<$crate::game::cards::Cards<2>> {
                &mut self.$base_field.hand
            }
            fn currency(&self) -> &$crate::currency::Currency {
                &self.$base_field.currency
            }
            fn currency_mut(&mut self) -> &mut $crate::currency::Currency {
                &mut self.$base_field.currency
            }
            $($extra)+
        }
        #[automatically_derived]
        unsafe impl Send for $struct {}
        #[automatically_derived]
        unsafe impl Sync for $struct {}
    };
}

impl Player {
    #[must_use]
    #[inline]
    pub fn show_hand(&self) -> String {
        show_cards(&self.hand())
    }

    pub fn new(hand: Cards<2>, lobby_seat: Seat) -> Self {
        let mut p = Self {
            state: Default::default(),
            total_bet: Default::default(),
            round_bet: Default::default(),
            seat: lobby_seat,
        };
        p.set_hand(hand);
        p
    }

    #[inline]
    pub fn set_hand(&mut self, hand: Cards<2>) {
        self.seat.behavior_mut().set_hand(hand);
    }

    #[inline]
    pub fn hand(&self) -> [Card; 2] {
        self.seat
            .behavior()
            .hand()
            .expect("hand of player was empty")
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
        *self.seat.behavior().currency()
    }
}
