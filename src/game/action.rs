use std::fmt::Display;

use tracing::{error, info};

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

    /// Helper: Get raise action (call + the amount you want to raise)
    pub fn action_raise(&self, bet_over_call: Currency) -> Action {
        let highest_bet = self.highest_bet_of_round();
        let player = self.current_player();
        let call_amount = highest_bet - player.round_bet;
        Action::Raise(call_amount + bet_over_call)
    }

    pub fn process_action(&mut self, action: Option<Action>) -> Result<()> {
        if self.is_finished() {
            return Err(PoksError::GameFinished);
        }
        let active_players: Vec<PlayerID> = self
            .players
            .iter()
            .enumerate()
            .filter(|(_, p)| p.state.is_playing())
            .map(|(id, _)| id)
            .collect();

        trace!("Active players: {}", active_players.len());
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
            info!("current player is not playing, skipping them");
            debug!("current_player.state={}", current_player.state());

            // since this player is not playing, self.active_players should not contain them
            debug_assert_eq!(
                self.active_players().position(|p| p == current_player),
                None
            );

            self.advance_turn()?;
            return Ok(());
        }

        let action = match action {
            Some(a) => a,
            None => {
                trace!("Player made no action, waiting for them");
                std::thread::sleep(std::time::Duration::from_millis(10));
                return Ok(());
            }
        };

        self.apply_action(action)?;

        if self.is_betting_complete() {
            info!("betting complete");
            // Move to next phase
            self.advance_phase()
        } else {
            info!("next turn");
            // Continue betting round
            self.advance_turn()
        }
    }

    // TODO: There is need for some account datastructure to take care of all these currencies
    // - The transactions here should likely be logged somewhere
    // - It should be checked that the player has enough currency
    fn apply_action(&mut self, action: Action) -> Result<()> {
        debug!("Applying action: {action}");
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
                    let delta = player.withdraw_currency(amount)?;
                    *player.round_bet_mut() += delta;
                }
            }
            Action::Raise(amount) => {
                let call_amount = highest_bet - player.round_bet;

                if state == GameState::RaiseDisallowed {
                    return Err(PoksError::RaiseNotAllowed);
                } else if amount > player.currency() {
                    return Err(PoksError::insufficient_funds(amount, player.currency()));
                } else if amount < min_raise {
                    error!(
                        "Player tried to raise by an amount less than the call + min raise: amount: {amount}, call_amount: {call_amount}"
                    );
                    return Err(crate::PoksError::raise_too_small(
                        amount,
                        min_raise + call_amount,
                    ));
                } else {
                    let delta = player.withdraw_currency(amount + call_amount)?;
                    *player.round_bet_mut() += delta;
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
                    let delta = player.withdraw_currency(amount)?;
                    *player.round_bet_mut() += delta;
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
