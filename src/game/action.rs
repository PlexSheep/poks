use std::fmt::Display;

use crate::{CU, currency::Currency};

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum Action {
    Fold,
    Call(Currency),
    Raise(Currency),
    AllIn(Currency),
}

impl Action {
    #[inline]
    pub fn check() -> Self {
        Self::Call(CU!(0))
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
