use std::fmt::{Debug, Display};

use crate::world::World;

struct Shortened;
impl Debug for Shortened {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "(..)")
    }
}

impl Debug for World {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("World")
            .field("evaluator", &Shortened)
            .field("players", &self.players)
            .field("game", &self.game)
            .field("deck", &self.deck)
            .field("action_log", &self.action_log)
            .finish()
    }
}
