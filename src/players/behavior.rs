use std::fmt::Debug;

use crate::{
    Result,
    currency::Currency,
    game::{Action, Game, cards::Cards},
};

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
