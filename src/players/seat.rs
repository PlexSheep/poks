use std::{
    any::Any as _,
    cmp::Ordering,
    ops::Deref,
    sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard},
};

use crate::{
    Result,
    currency::Currency,
    game::Action,
    players::{BehaveBox, Player, PlayerBehavior},
};

#[derive(Debug, Clone)]
#[must_use]
pub struct Seat {
    currency: Arc<RwLock<Currency>>,
    behavior: Arc<RwLock<BehaveBox>>,
}

impl Seat {
    pub fn new<B>(starting_cash: Currency, behavior: B) -> Self
    where
        B: PlayerBehavior + Send + Sync + 'static,
    {
        Self {
            currency: Arc::new(RwLock::new(starting_cash)),
            behavior: Arc::new(RwLock::new(Box::new(behavior))),
        }
    }

    #[inline]
    pub fn behavior<'a>(&'a self) -> RwLockReadGuard<'a, BehaveBox> {
        self.behavior
            .read()
            .expect("could not access player behavior of lobby seat")
    }

    #[inline]
    pub fn behavior_mut<'a>(&'a self) -> RwLockWriteGuard<'a, BehaveBox> {
        self.behavior
            .write()
            .expect("could not access player behavior of lobby seat")
    }

    pub fn set_currency(&mut self, cu: Currency) {
        *self.currency_mut() = cu;
    }

    fn currency_mut(&mut self) -> std::sync::RwLockWriteGuard<'_, Currency> {
        self.currency
            .write()
            .expect("could not get seat's currency for writing")
    }

    #[inline]
    pub fn add_currency(&mut self, cu: Currency) -> Result<()> {
        *self.currency_mut() += cu;
        Ok(())
    }

    #[inline]
    pub fn withdraw_currency(&mut self, cu: Currency) -> Result<Currency> {
        if self.currency() < cu {
            Err(crate::PoksError::TooLittleCurrency)
        } else {
            *self.currency_mut() -= cu;
            Ok(cu)
        }
    }

    pub fn currency(&self) -> Currency {
        *self
            .currency
            .read()
            .expect("could not get seat's currency for reading")
    }

    pub fn act(&self, game: &crate::game::Game, player: &Player) -> Result<Option<Action>> {
        self.behavior_mut().act(game, player)
    }

    fn behavior_typeid(&self) -> std::any::TypeId {
        self.behavior().deref().deref().type_id()
    }
}

impl PartialEq for Seat {
    fn eq(&self, other: &Self) -> bool {
        self.currency().cmp(&other.currency()) == Ordering::Equal
            && self.behavior_typeid().cmp(&other.behavior_typeid()) == Ordering::Equal
    }
}

impl Eq for Seat {}
