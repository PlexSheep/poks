use std::sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard};

use tracing::trace;

use crate::{
    Result,
    currency::Currency,
    game::Action,
    players::{BehaveBox, Player, PlayerBehavior},
};

#[derive(Debug, Clone)]
pub struct Seat {
    currency: Currency,
    behavior: Arc<RwLock<BehaveBox>>,
}

impl Seat {
    pub fn new<B>(starting_cash: Currency, behavior: B) -> Self
    where
        B: PlayerBehavior + Send + Sync + 'static,
    {
        Self {
            currency: starting_cash,
            behavior: Arc::new(RwLock::new(Box::new(behavior))),
        }
    }

    pub fn new_box(
        starting_cash: Currency,
        behavior: Box<dyn PlayerBehavior + Send + Sync>,
    ) -> Self {
        Self {
            currency: starting_cash,
            behavior: Arc::new(RwLock::new(behavior)),
        }
    }

    #[inline]
    pub fn behavior<'a>(&'a self) -> RwLockReadGuard<'a, BehaveBox> {
        trace!("get seat behavior");
        self.behavior
            .read()
            .expect("could not access player behavior of lobby seat")
    }

    #[inline]
    pub fn behavior_mut<'a>(&'a self) -> RwLockWriteGuard<'a, BehaveBox> {
        trace!("get seat behavior_mut");
        self.behavior
            .write()
            .expect("could not access player behavior of lobby seat")
    }

    pub fn set_currency(&mut self, cu: Currency) {
        self.currency = cu;
    }

    pub fn currency_mut(&mut self) -> &mut Currency {
        &mut self.currency
    }

    pub fn currency(&self) -> Currency {
        self.currency
    }

    pub fn act(&self, game: &crate::game::Game, player: &Player) -> Result<Option<Action>> {
        self.behavior_mut().act(game, player)
    }
}

unsafe impl Send for Seat {}
unsafe impl Sync for Seat {}
