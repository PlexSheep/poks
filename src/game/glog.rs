use crate::players::PlayerID;

mod macros {
    macro_rules! glog {
        ($self:tt, None, $stuff:expr) => {
            $self.game_log.push((None, $stuff))
        };
        ($self:tt, $player:expr, $stuff:expr) => {
            $self.game_log.push((Some($player), $stuff))
        };
    }

    macro_rules! glogf {
    ($self:tt, None, $($content:tt)+) => {
        $self.game_log.push((None, format!($($content)+)))
    };
    ($self:tt, $player:expr, $($content:tt)+) => {
        $self.game_log.push((Some($player), format!($($content)+)))
    };
}
    pub(crate) use {glog, glogf};
}
pub(crate) use macros::*;

pub type GlogItem = (Option<PlayerID>, String);

impl super::Game {
    pub fn gamelog(&self) -> &[GlogItem] {
        &self.game_log
    }

    pub fn take_gamelog(&mut self) -> Vec<GlogItem> {
        let a = self.game_log.clone();
        self.game_log = Vec::with_capacity(32);
        a
    }
}
