use std::fmt::Debug;

use crate::game::World;

#[derive(Debug)]
struct Shortened;

impl Debug for World {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("World")
            .field("evaluator", &Shortened)
            .field("player_amount", &self.player_amount)
            .finish()
    }
}
