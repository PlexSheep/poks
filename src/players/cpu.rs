use rand::prelude::*;

use crate::{
    CU, Result,
    game::{Action, Game},
    player_impl,
    players::PlayerBasicFields,
};

#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord, Default)]
pub struct PlayerCPU {
    base: PlayerBasicFields,
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
