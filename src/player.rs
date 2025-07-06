use std::sync::atomic::AtomicBool;
use std::{fmt::Debug, sync::RwLock};

use crate::currency::Currency;
use crate::game::{Action, Cards, Game};

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord, Default)]
pub enum PlayerState {
    #[default]
    Playing,
    Folded,
    Paused,
    Lost,
}

pub trait PlayerBehavior: Debug {
    fn hand(&self) -> &Option<Cards<2>>;
    fn hand_mut(&mut self) -> &mut Option<Cards<2>>;
    fn currency(&self) -> &Currency;
    fn currency_mut(&mut self) -> &mut Currency;
    fn act(&self, game: &Game) -> Action;

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
pub struct PlayerLocal {
    hand: Option<Cards<2>>,
    currency: Currency,
}

#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord, Default)]
pub struct PlayerCPU {
    hand: Option<Cards<2>>,
    currency: Currency,
}

pub static LOCAL_USER_ACTION_READY: AtomicBool = AtomicBool::new(false);
pub static LOCAL_USER_ACTION: RwLock<Action> = RwLock::new(Action::Check);

macro_rules! player_impl {
    ($struct:ident, $($extra:tt)+) => {
        impl PlayerBehavior for $struct {
            fn hand(&self) -> &Option<Cards<2>> {
                &self.hand
            }

            fn hand_mut(&mut self) -> &mut Option<Cards<2>> {
                &mut self.hand
            }
            fn currency(&self) -> &Currency {
                &self.currency
            }
            fn currency_mut(&mut self) -> &mut Currency {
                &mut self.currency
            }
            $($extra)+
        }
    };
}

player_impl!(
    PlayerCPU,
    fn act(&self, _game: &Game) -> Action {
        let a = rand::random();
        match a {
            Action::Raise(bet) => {
                if self.currency < bet {
                    return Action::Check;
                }
                a
            }
            a => a,
        }
    }
);
