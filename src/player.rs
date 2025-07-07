use rand::prelude::*;
use std::fmt::Debug;

use crate::currency::Currency;
use crate::game::{Action, Cards, Game};
use crate::{CU, Result};

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
    // TODO: add some functionality to ensure this isn't called too often, since it might be
    // compute heavy
    fn act(&mut self, game: &Game) -> Result<Option<Action>>;

    fn set_hand(&mut self, new: Cards<2>) {
        *self.hand_mut() = Some(new);
    }
    fn set_currency(&mut self, new: Currency) {
        *self.currency_mut() = new;
    }
    fn win(&mut self, game: Game) {
        *self.currency_mut() += game.pot();
    }
}

#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord, Default)]
pub struct PlayerBasicFields {
    pub hand: Option<Cards<2>>,
    pub currency: Currency,
}

#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord, Default)]
pub struct PlayerCPU {
    base: PlayerBasicFields,
}

#[macro_export]
macro_rules! player_impl {
    ($struct:ident, $base_field:tt, $($extra:tt)+) => {
        impl $crate::player::PlayerBehavior for $struct {
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
    };
}

player_impl!(
    PlayerCPU,
    base,
    fn act(&mut self, game: &Game) -> Result<Option<Action>> {
        let mut rng = rand::rngs::OsRng;
        let disc: u8 = rng.gen_range(0..=100);
        let mut a = match disc {
            0..10 => Action::Fold,
            10..70 => game.action_call(),
            70..99 => Action::Raise(CU!(10)),
            99 => Action::Raise(CU!(100)),
            100 => Action::AllIn(*self.currency()),
            _ => unreachable!(),
        };

        if let Action::Raise(bet) = a {
            if bet >= *self.currency() {
                a = Action::Fold;
            }
        }

        Ok(Some(a))
    }
);

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
