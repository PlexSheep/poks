pub mod cpu;
pub mod local;

pub use cpu::PlayerCPU;
pub use local::PlayerLocal;

use std::fmt::Debug;

use crate::Result;
use crate::currency::Currency;
use crate::game::{Action, Cards, Game};

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord, Default)]
pub enum PlayerState {
    #[default]
    Playing,
    AllIn,
    Folded,
    Paused,
    Lost,
}

pub trait PlayerBehavior: Debug {
    fn hand(&self) -> &Option<Cards<2>>;
    fn hand_mut(&mut self) -> &mut Option<Cards<2>>;
    fn currency(&self) -> &Currency;
    fn currency_mut(&mut self) -> &mut Currency;
    fn act(&mut self, game: &Game) -> Result<Option<Action>>;

    #[inline]
    fn set_hand(&mut self, new: Cards<2>) {
        *self.hand_mut() = Some(new);
    }
    #[inline]
    fn set_currency(&mut self, new: Currency) {
        *self.currency_mut() = new;
    }
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
            fn hand(&self) -> &Option<$crate::game::Cards<2>> {
                &self.$base_field.hand
            }

            fn hand_mut(&mut self) -> &mut Option<$crate::game::Cards<2>> {
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

impl PlayerState {
    #[inline]
    #[must_use]
    pub fn is_playing(&self) -> bool {
        match self {
            PlayerState::Playing | PlayerState::AllIn => true,
            PlayerState::Folded | PlayerState::Paused | PlayerState::Lost => false,
        }
    }
}
