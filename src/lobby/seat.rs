use std::sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard};

use tracing::trace;

use crate::{
    currency::Currency,
    game::cards::Cards,
    players::{BehaveBox, PlayerBehavior},
};

#[derive(Debug, Clone)]
pub struct Seat {
    inner: Arc<RwLock<BehaveBox>>,
}

impl Seat {
    pub fn new(behavior: Box<dyn PlayerBehavior + Send + Sync>) -> Self {
        Self {
            inner: Arc::new(RwLock::new(behavior)),
        }
    }

    #[inline]
    pub fn behavior<'a>(&'a self) -> RwLockReadGuard<'a, BehaveBox> {
        trace!("get seat behavior");
        self.inner
            .read()
            .expect("could not access player behavior of lobby seat")
    }

    #[inline]
    pub fn behavior_mut<'a>(&'a self) -> RwLockWriteGuard<'a, BehaveBox> {
        trace!("get seat behavior_mut");
        self.inner
            .write()
            .expect("could not access player behavior of lobby seat")
    }

    pub fn currency(&self) -> Currency {
        *self.behavior().currency()
    }

    pub fn hand(&self) -> Option<Cards<2>> {
        *self.behavior().hand()
    }

    pub fn set_currency(&self, cu: Currency) {
        self.behavior_mut().set_currency(cu);
    }
}

impl From<BehaveBox> for Seat {
    fn from(value: BehaveBox) -> Self {
        Self::new(value)
    }
}

unsafe impl Send for Seat {}
unsafe impl Sync for Seat {}
