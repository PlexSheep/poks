use std::sync::{Arc, RwLock};

use poks::Result;
use poks::game::{Action, Game};
use poks::{player::PlayerBasicFields, player_impl};

pub type ActionAccessor = Arc<RwLock<Option<Action>>>;

#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord, Default)]
pub struct PlayerLocal {
    pub base: PlayerBasicFields,
    pub next_action: ActionAccessor,
}

impl PlayerLocal {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn action_field_reference(&self) -> Arc<RwLock<Option<Arc>>> {
        self.next_action.clone()
    }

    pub fn set_action(accessor: ActionAccessor, action: Action) {
        *accessor.write() = action;
    }

    pub fn get_action(accessor: ActionAccessor, action: Action) -> Action {
        *accessor.read()
    }
}

player_impl!(
    PlayerLocal,
    base,
    fn act(&mut self, _game: &Game) -> Result<Option<Action>> {
        Ok(self.next_action.take())
    }
);
