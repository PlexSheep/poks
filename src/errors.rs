use crate::currency::Currency;
use crate::players::{PlayerID, PlayerState};
use thiserror::Error;

pub type Result<T> = std::result::Result<T, PoksError>;

#[derive(Error, Debug)]
pub enum PoksError {
    // Game Logic Errors
    #[error("Game is already finished")]
    GameFinished,

    #[error("Game has not started yet")]
    GameNotStarted,

    #[error("Invalid player ID: {player_id} (max: {max_players})")]
    InvalidPlayerId {
        player_id: PlayerID,
        max_players: usize,
    },

    #[error("Player {player_id} is not in a playing state (current state: {current_state})")]
    PlayerNotPlaying {
        player_id: PlayerID,
        current_state: PlayerState,
    },

    #[error("Player {player_id} is already all-in")]
    PlayerAlreadyAllIn { player_id: PlayerID },

    #[error("Not enough players to start game (need at least 2, have {count})")]
    InsufficientPlayers { count: usize },

    #[error("Too many players for deck (requested: {requested}, max supported: {max})")]
    TooManyPlayers { requested: usize, max: usize },

    // Action/Betting Errors
    #[error("Invalid action: cannot call when you're not under the round bet")]
    InvalidCall,

    // Action/Betting Errors
    #[error("Invalid call amount: expected {expected}, got {actual}")]
    CallAmountMismatch {
        expected: Currency,
        actual: Currency,
    },

    #[error("Cannot raise: betting is not allowed in current game state")]
    RaiseNotAllowed,

    #[error("Insufficient funds: need {required}, have {available}")]
    InsufficientFunds {
        required: Currency,
        available: Currency,
    },

    #[error("Invalid bet amount: {amount} (minimum: {minimum})")]
    TooLowBetAmount { amount: Currency, minimum: Currency },

    // Card/Deck Errors
    #[error("Not enough cards in deck")]
    InsufficientCards,

    #[error("Card evaluation failed: {reason}")]
    CardEvaluationError { reason: String },

    // Transaction Errors
    #[error("Transaction failed: {reason}")]
    TransactionError { reason: String },

    #[error("Currency overflow in transaction")]
    CurrencyOverflow,

    // World/Player Management Errors
    #[error("Player action timeout")]
    PlayerTimeout,

    #[error("Failed to add player: {reason}")]
    PlayerAddError { reason: String },

    #[error("World is in invalid state: {reason}")]
    InvalidWorldState { reason: String },

    // UI/TUI Specific Errors
    #[error("Terminal initialization failed")]
    TerminalError,

    #[error("Failed to handle terminal event: {event}")]
    EventHandlingError { event: String },

    // IO and External Errors
    #[error("File operation failed")]
    IoError(#[from] std::io::Error),

    #[error("Logging setup failed")]
    LoggingError,

    // Generic errors for unexpected situations
    #[error("Internal error: {message}")]
    Internal { message: String },

    #[error("Configuration error: {field} - {reason}")]
    ConfigError { field: String, reason: String },
}

mod macros {
    macro_rules! err_int {
    ($($toks:tt)+) => {
        $crate::errors::PoksError::internal(format!($($toks)+))
    };
}
    pub(crate) use err_int;
}
pub(crate) use macros::*;

impl PoksError {
    // Convenience constructors for common error patterns
    pub fn insufficient_funds(required: Currency, available: Currency) -> Self {
        Self::InsufficientFunds {
            required,
            available,
        }
    }

    pub fn invalid_player(player_id: PlayerID, max_players: usize) -> Self {
        Self::InvalidPlayerId {
            player_id,
            max_players,
        }
    }

    pub fn player_not_playing(player_id: PlayerID, current_state: PlayerState) -> Self {
        Self::PlayerNotPlaying {
            player_id,
            current_state,
        }
    }

    pub fn internal(message: impl Into<String>) -> Self {
        Self::Internal {
            message: message.into(),
        }
    }

    pub fn card_evaluation(reason: impl Into<String>) -> Self {
        Self::CardEvaluationError {
            reason: reason.into(),
        }
    }

    pub fn call_mismatch(expected: Currency, actual: Currency) -> Self {
        Self::CallAmountMismatch { expected, actual }
    }

    pub fn too_many_players(requested: usize, max: usize) -> Self {
        Self::TooManyPlayers { requested, max }
    }
}

// Helper trait for adding context to results
pub trait PoksErrorExt<T> {
    fn with_context(self, context: impl Into<String>) -> Result<T>;
    fn with_player_context(self, player_id: PlayerID) -> Result<T>;
}

impl<T, E> PoksErrorExt<T> for std::result::Result<T, E>
where
    E: Into<PoksError>,
{
    fn with_context(self, context: impl Into<String>) -> Result<T> {
        self.map_err(|_| PoksError::internal(context.into()))
    }

    fn with_player_context(self, player_id: PlayerID) -> Result<T> {
        self.map_err(|_| PoksError::internal(format!("Error with player {player_id}")))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::CU;

    #[test]
    fn test_error_display() {
        let error = PoksError::insufficient_funds(CU!(1), CU!(0, 50));
        assert_eq!(
            error.to_string(),
            "Insufficient funds: need 1,00ŧ, have 0,50ŧ"
        );
    }

    #[test]
    fn test_error_convenience_constructors() {
        let error = PoksError::invalid_player(5, 4);
        assert!(matches!(
            error,
            PoksError::InvalidPlayerId {
                player_id: 5,
                max_players: 4
            }
        ));
    }
}
