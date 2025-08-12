use rand::prelude::*;
use tracing::{debug, warn};

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
            70..99 if player.currency() > CU!(50) => Action::Raise(CU!(10)),
            99 if player.currency() > CU!(500) => Action::Raise(CU!(100)),
            // 100 => Action::AllIn(player.currency()),
            _ => {
                warn!("CPU Player random action generator is out of range, defaulting to call");
                game.action_call()
            }
        };

        if let Action::Raise(bet) = a {
            if bet >= player.currency() {
                a = Action::Fold;
            }
        }

        debug!("CPU Player acts: {a}");
        Ok(Some(a))
    }
}
