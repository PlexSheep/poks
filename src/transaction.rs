use std::ops::{Deref, DerefMut};

use crate::Result;
use crate::currency::Currency;

static mut GARBAGE: Currency = Currency::new(0, 0);

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord, Default)]
#[must_use]
pub struct Transaction {
    amount: Currency,
}

impl Transaction {
    pub fn new(amount: Currency) -> Self {
        Transaction { amount }
    }
    pub fn amount(&self) -> Currency {
        self.amount
    }
    pub fn finish(self, sender: &mut Currency, receiver: &mut Currency) -> Result<()> {
        *sender -= self.amount;
        *receiver += self.amount;
        Ok(())
    }

    #[allow(static_mut_refs)]
    /// # Safety
    ///
    /// Reading from this reference is undefined behavior
    pub fn garbage() -> &'static mut Currency {
        unsafe { &mut GARBAGE }
    }
}

impl Deref for Transaction {
    type Target = Currency;

    fn deref(&self) -> &Self::Target {
        &self.amount
    }
}

impl DerefMut for Transaction {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.amount
    }
}
