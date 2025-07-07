use std::sync::{Arc, RwLock};

use crate::Result;
use crate::game::{Action, Game};
use crate::{player_impl, players::PlayerBasicFields};

pub type ActionAccessor = Arc<RwLock<Option<Action>>>;

#[derive(Debug, Clone, Default)]
pub struct PlayerLocal {
    pub base: PlayerBasicFields,
    pub next_action: ActionAccessor,
}

impl PlayerLocal {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn action_field_reference(&self) -> Arc<RwLock<Option<Action>>> {
        self.next_action.clone()
    }

    pub fn set_action(accessor: &ActionAccessor, action: Action) {
        *accessor
            .write()
            .expect("could not read from local player accessor") = Some(action);
    }

    pub fn get_action(accessor: &ActionAccessor) -> Option<Action> {
        *accessor
            .read()
            .expect("could not read from local player accessor")
    }

    fn take_next_action(&self) -> Option<Action> {
        self.next_action
            .write()
            .expect("could not read from local player accessor")
            .take()
    }
}

player_impl!(
    PlayerLocal,
    base,
    fn act(&mut self, _game: &Game) -> Result<Option<Action>> {
        Ok(self.take_next_action())
    }
);
