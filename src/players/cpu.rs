use rand::prelude::*;

use crate::{
    CU, Result,
    game::{Action, Game},
    players::{PlayerBasicFields, PlayerBehavior},
};

#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord, Default)]
pub struct PlayerCPU {
    base: PlayerBasicFields,
}

impl PlayerBehavior for PlayerCPU {
    fn act(&mut self, game: &Game, player: &super::Player) -> Result<Option<Action>> {
        let mut rng = rand::rngs::OsRng;
        let disc: u8 = rng.gen_range(0..=100);
        let mut a = match disc {
            0..10 => Action::Fold,
            10..70 => game.action_call(),
            70..99 => game.action_raise(CU!(10)),
            99 => Action::Raise(CU!(100)),
            100 => Action::AllIn(player.currency()),
            _ => unreachable!(),
        };

        if let Action::Raise(bet) = a {
            if bet >= player.currency() {
                a = Action::Fold;
            }
        }

        Ok(Some(a))
    }
}
