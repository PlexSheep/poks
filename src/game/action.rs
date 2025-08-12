use std::fmt::Display;

use tracing::{error, info, warn};

use super::*;
use crate::{CU, PoksError, currency::Currency, game::glogf};

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

        if !self.has_enough_active_players() {
            warn!("Processing action but not enough players active");
            return Ok(());
        };

        let pid = self.current_player_id();

        if self.current_player().currency() == CU!(0) {
            self.current_player_mut().set_state(PlayerState::Lost);
            // FIXME: somehow glogf! does not work here

            glogf!(self, pid, "is bankrupt.");
        }

        let current_player = self.current_player();

        // skip players who are not actually playing
        // NOTE: might need extra logic for all in players here?
        if !current_player.state().is_playing() {
            info!("current player is not playing, skipping them");
            debug!("current_player.state={}", current_player.state());

            // since this player is not playing, self.active_players should not contain them
            debug_assert_eq!(
                self.active_players()
                    .position(|p| p == self.current_player()),
                None
            );

            self.advance_turn()?;
            return Ok(());
        }

        let action = match action {
            Some(a) => a,
            None => {
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
        let pid = self.current_player_id();
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
                    glogf!(self, pid, "calls for {delta}");
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
                    glogf!(self, pid, "raises by {delta}");
                }
            }
            Action::AllIn(amount) => {
                if amount != player.currency() {
                    return Err(PoksError::BetAmountMismatch {
                        expected: player.currency(),
                        actual: amount,
                    });
                } else {
                    player.set_state(PlayerState::AllIn);
                    let delta = player.withdraw_currency(amount)?;
                    *player.round_bet_mut() += delta;
                    glogf!(self, pid, "goes all in with {delta}!");
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
