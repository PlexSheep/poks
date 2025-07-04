use std::sync::atomic::AtomicBool;
use std::{fmt::Debug, sync::RwLock};

use crate::game::{Action, Currency, Game, Hand};

#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum Player {
    Local(PlayerLocal),
    CPU(PlayerCPU),
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum PlayerState {
    Playing,
    Folded,
    Paused,
    Lost,
}

pub trait PlayerBehavior {
    fn hand(&self) -> &Option<Hand>;
    fn hand_mut(&mut self) -> &mut Option<Hand>;
    fn currency(&self) -> &Currency;
    fn currency_mut(&mut self) -> &mut Currency;
    fn act(&self, game: &Game) -> Action;

    fn set_hand(&mut self, new: Hand) {
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
    hand: Option<Hand>,
    currency: Currency,
}

#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord, Default)]
pub struct PlayerCPU {
    hand: Option<Hand>,
    currency: Currency,
}

pub static LOCAL_USER_ACTION_READY: AtomicBool = AtomicBool::new(false);
pub static LOCAL_USER_ACTION: RwLock<Action> = RwLock::new(Action::Check);

macro_rules! call_enum_functions {
    ($self:expr,$($function:tt)+) => {
        match $self {
            Player::Local(p) => p.$($function)+,
            Player::CPU(p) => p.$($function)+,
        }
    };
}

impl PlayerBehavior for Player {
    fn hand(&self) -> &Option<Hand> {
        call_enum_functions!(self, hand())
    }

    fn hand_mut(&mut self) -> &mut Option<Hand> {
        call_enum_functions!(self, hand_mut())
    }

    fn currency(&self) -> &Currency {
        call_enum_functions!(self, currency())
    }

    fn currency_mut(&mut self) -> &mut Currency {
        call_enum_functions!(self, currency_mut())
    }
    fn act(&self, game: &Game) -> Action {
        call_enum_functions!(self, act(game))
    }
}

macro_rules! player_impl {
    ($struct:ident, $($extra:tt)+) => {
        impl PlayerBehavior for $struct {
            fn hand(&self) -> &Option<Hand> {
                &self.hand
            }

            fn hand_mut(&mut self) -> &mut Option<Hand> {
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

impl PlayerLocal {
    pub fn set_action_is_ready(ready: bool) {
        LOCAL_USER_ACTION_READY.store(ready, std::sync::atomic::Ordering::Relaxed);
    }
    pub fn get_action_is_ready() -> bool {
        LOCAL_USER_ACTION_READY.load(std::sync::atomic::Ordering::Relaxed)
    }
    pub fn set_action(action: Action) {
        *LOCAL_USER_ACTION
            .write()
            .expect("could not read local user action") = action;
    }
    pub fn get_action() -> Action {
        assert!(Self::get_action_is_ready());
        *LOCAL_USER_ACTION
            .read()
            .expect("could not read local user action")
    }
}

player_impl!(
    PlayerLocal,
    fn act(&self, _game: &Game) -> Action {
        // HACK: this is horrible from design, I should have some way to pass an argument to this
        // from the ui!

        if !Self::get_action_is_ready() {
            return Action::HiddenWait;
        }
        let a = Self::get_action();
        Self::set_action_is_ready(false);
        a
    }
);
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

impl Player {
    pub const fn local(currency: Currency) -> Self {
        Self::Local(PlayerLocal {
            hand: None,
            currency,
        })
    }
    pub const fn cpu(currency: Currency) -> Self {
        Self::CPU(PlayerCPU {
            hand: None,
            currency,
        })
    }
}
