#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord, Default)]
#[non_exhaustive]
pub enum GameState {
    #[default]
    RaiseAllowed,
    RaiseDisallowed,
    Pause,
    Finished,
}

impl GameState {
    #[inline]
    #[must_use]
    pub fn is_ongoing(&self) -> bool {
        match self {
            GameState::RaiseAllowed | GameState::RaiseDisallowed => true,
            GameState::Pause | GameState::Finished => false,
        }
    }
}
