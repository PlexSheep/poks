use std::fmt::Debug;

use crate::{
    Result,
    game::{Action, Game},
    players::Player,
};

pub trait PlayerBehavior: Debug {
    fn act(&mut self, game: &Game, player: &Player) -> Result<Option<Action>>;
}
