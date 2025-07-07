use std::fmt::Display;

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord, Default)]
pub enum PlayerState {
    #[default]
    Playing,
    AllIn,
    Folded,
    Paused,
    Lost,
}

impl PlayerState {
    #[inline]
    #[must_use]
    pub fn is_playing(&self) -> bool {
        match self {
            PlayerState::Playing | PlayerState::AllIn => true,
            PlayerState::Folded | PlayerState::Paused | PlayerState::Lost => false,
        }
    }
}

impl Display for PlayerState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}
