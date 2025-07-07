use std::fmt::Display;

use super::evaluation::show_eval_cards;

use tracing::{debug, info};

use super::Game;
use super::cards::Cards;
use super::evaluation::{Eval, FiveCard};
use crate::{CU, Result, currency::Currency, players::PlayerID};

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum Winner {
    UnknownCards(Currency, PlayerID),
    KnownCards(Currency, PlayerID, Eval<FiveCard>, Cards<7>),
}

impl Winner {
    pub fn payout(&self, game: &mut Game) -> Result<()> {
        info!("Payout!");
        let winnings = game.pot();
        let player = &mut game.players[self.pid()];
        let old = player.currency();
        assert_ne!(winnings, CU!(0));
        *player.currency_mut() += winnings;
        assert_eq!(old + winnings, player.currency());
        debug!("After Payout? {}", player.currency());
        Ok(())
    }

    pub fn pid(&self) -> PlayerID {
        match self {
            Winner::UnknownCards(_, pid) => *pid,
            Winner::KnownCards(_, pid, ..) => *pid,
        }
    }
}

impl Display for Winner {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::KnownCards(pot, pid, eval, cards) => {
                    format!(
                        "Player {pid} won {pot} with {eval}:\n  {}",
                        show_eval_cards(eval.classify(), cards)
                    )
                }
                Self::UnknownCards(pot, pid) => format!("Player {pid} won {pot}."),
            }
        )
    }
}
