use std::fmt::Display;

use super::*;
use crate::{CU, PoksError, currency::Currency};

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum Action {
    Fold,
    Call(Currency),
    Raise(Currency),
    AllIn(Currency),
}

impl Action {
    /// Create a check action (call for 0 when no bet exists)
    #[inline]
    pub fn check() -> Self {
        Self::Call(CU!(0))
    }
}

impl super::Game {
    /// Helper: Get call action (check if no bet, call the difference otherwise)
    pub fn action_call(&self) -> Action {
        let diff = self.highest_bet_of_round() - self.players[self.turn].round_bet;
        Action::Call(diff)
    }

    pub fn process_action(&mut self, action: Option<Action>) -> Result<()> {
        let active_players: Vec<PlayerID> = self
            .players
            .iter()
            .enumerate()
            .filter(|(_, p)| p.state.is_playing())
            .map(|(id, _)| id)
            .collect();

        if active_players.len() <= 1 {
            if let Some(winner_id) = active_players.first() {
                debug!("Player action was dropped because they are the only player left");
                self.set_winner(Winner::UnknownCards(self.pot(), *winner_id));
                return Ok(());
            } else {
                return Err(crate::PoksError::NoActivePlayers);
            }
        }

        let current_player = self.current_player();

        // skip players who are not actually playing
        // NOTE: might need extra logic for all in players here?
        if !current_player.state().is_playing() {
            self.advance_turn()?;
            return Ok(());
        }

        // Wait for action if none provided
        let action = match action {
            Some(a) => a,
            None => return Ok(()), // Come back when they have an action
        };

        self.apply_action(action)?;

        if self.is_betting_complete() {
            // Move to next phase
            self.advance_phase()
        } else {
            // Continue betting round
            self.advance_turn()
        }
    }

    // TODO: There is need for some account datastructure to take care of all these currencies
    // - The transactions here should likely be logged somewhere
    // - It should be checked that the player has enough currency
    fn apply_action(&mut self, action: Action) -> Result<()> {
        let highest_bet = self.highest_bet_of_round();
        let state = self.state;
        let min_raise = self.min_raise_amount();
        let player = self.current_player_mut();

        match action {
            Action::Fold => {
                player.set_state(PlayerState::Folded);
            }
            Action::Call(amount) => {
                let needed = highest_bet - player.round_bet;
                if amount != needed {
                    return Err(PoksError::BetAmountMismatch {
                        expected: needed,
                        actual: amount,
                    });
                } else if amount > player.currency() {
                    return Err(PoksError::insufficient_funds(amount, player.currency()));
                } else {
                    *player.round_bet_mut() += amount;
                    *player.currency_mut() -= amount;
                }
            }
            Action::Raise(amount) => {
                let call_amount = highest_bet - player.round_bet;
                let plus_call = amount - call_amount;

                if state == GameState::RaiseDisallowed {
                    return Err(PoksError::RaiseNotAllowed);
                } else if amount > player.currency() {
                    return Err(PoksError::insufficient_funds(amount, player.currency()));
                } else if plus_call > min_raise {
                    return Err(crate::PoksError::RaiseTooSmall(amount, min_raise));
                } else {
                    *player.round_bet_mut() += amount;
                    *player.currency_mut() -= amount;
                }
            }
            Action::AllIn(amount) => {
                if amount != player.currency() {
                    return Err(PoksError::BetAmountMismatch {
                        expected: player.currency(),
                        actual: amount,
                    });
                } else {
                    player.state = PlayerState::AllIn;
                    *player.round_bet_mut() += amount;
                    *player.currency_mut() -= amount;
                }
            }
        }
        Ok(())
    }
}

impl Display for Action {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Action::Fold => "folds".to_string(),
                Action::Call(bet) if *bet == CU!(0) => "checks".to_string(),
                Action::Call(bet) => format!("calls for {bet}"),
                Action::Raise(bet) => format!("raises by {bet}"),
                Action::AllIn(bet) => format!("goes all in! ({bet})"),
            }
        )
    }
}
