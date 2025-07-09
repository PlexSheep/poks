use std::fmt::Debug;

use rand::prelude::*;
use tracing::{debug, trace};

mod action;
mod betting; // this one has most of the actual game functionality
pub mod cards;
pub mod evaluation;
mod glog;
mod phase;
mod state;
mod winner;

pub use action::*;
use cards::*;
use evaluation::*;
pub use glog::*;
pub use phase::*;
pub use state::*;
pub use winner::*;

use crate::currency::Currency;
use crate::players::{Player, PlayerID, PlayerState, Seat};
use crate::{CU, Result};

pub type RNG = rand::rngs::StdRng;
pub type Seed = <RNG as rand::SeedableRng>::Seed;

/// # Panics
///
/// Methods and associated functions of this struct will panic if you pass an invalid [`PlayerID`]
/// as argument.
#[derive(Debug, Clone)]
pub struct Game {
    phase: Phase,
    turn: PlayerID,
    dealer: PlayerID,
    players: Vec<Player>,
    community_cards: CardsDynamic,
    winner: Option<Winner>,
    deck: CardsDynamic,
    state: GameState,
    small_blind: Currency,
    big_blind: Currency,
    game_log: Vec<GlogItem>,
    #[allow(unused)]
    seed: Seed,
}

impl Game {
    pub fn seed() -> Seed {
        let mut os_rng = rand::rngs::OsRng;
        let mut seed: Seed = Seed::default();
        let mut guard = 0;
        while seed == Seed::default() {
            seed = os_rng.r#gen();
            guard += 1;
            if guard > 255 {
                panic!(
                    "Generating a seed failed 256 times in a row, something is wrong with the os rng!!!"
                )
            }
        }
        assert_ne!(seed, [0; 32]); // enough seeds besides that one.
        seed
    }

    pub fn buid_with_seed(seats: &[Seat], dealer_pos: PlayerID, seed: Seed) -> Result<Self> {
        trace!("Building a new game");
        assert!(seats.len() >= 2);
        let mut rng = RNG::from_seed(seed);
        let mut deck: CardsDynamic = poker::deck::shuffled_with(&mut rng).into();
        if seats.len() > deck.len() / 2 {
            // TODO: return a proper error and result
            panic!("Not enough cards in a deck for this many players!")
        }
        let mut players = Vec::new();
        for seat in seats {
            let hand: Cards<2> = [deck.pop().unwrap(), deck.pop().unwrap()];
            players.push(Player::new(hand, seat.clone()));
        }
        let mut game = Game {
            turn: 0,
            phase: Phase::default(),
            players,
            community_cards: CardsDynamic::new(),
            winner: None,
            deck,
            state: GameState::default(),
            small_blind: CU!(0, 50),
            big_blind: CU!(1),
            dealer: dealer_pos,
            game_log: Vec::with_capacity(32),
            seed,
        };

        game.post_blinds()?;

        trace!("New game is ready");
        Ok(game)
    }

    #[inline]
    pub fn build(seats: &[Seat], dealer_pos: PlayerID) -> Result<Self> {
        let seed = Self::seed();
        Self::buid_with_seed(seats, dealer_pos, seed)
    }

    #[inline]
    pub fn phase(&self) -> Phase {
        self.phase
    }

    #[inline]
    pub fn phase_mut(&mut self) -> &mut Phase {
        &mut self.phase
    }

    #[inline]
    pub fn set_phase(&mut self, phase: Phase) {
        for player in self.players.iter_mut() {
            player.total_bet += player.round_bet;
            player.round_bet = Currency::ZERO;
        }
        self.phase = phase;
        glogf!(self, None, "Phase: {phase}");
    }

    #[inline]
    pub fn is_finished(&self) -> bool {
        self.winner.is_some()
    }

    pub fn set_winner(&mut self, w: Winner) {
        w.payout(self).expect("could not payout the winner");
        self.winner = Some(w);
        glog!(self, None, self.winner.unwrap().to_string())
    }

    #[inline]
    pub fn winner(&self) -> Option<Winner> {
        self.winner
    }

    #[inline]
    pub(crate) fn draw_card(&mut self) -> Card {
        self.deck.pop().unwrap()
    }

    #[inline]
    fn add_table_card(&mut self) {
        let c = self.draw_card();
        self.community_cards.push(c);
    }

    pub fn hand_plus_table(&self, pid: PlayerID) -> CardsDynamic {
        let player = &self.players[pid];
        let mut hand_plus_table: CardsDynamic = player.hand().into();
        hand_plus_table.extend(self.community_cards.iter());
        hand_plus_table.sort();
        hand_plus_table
    }

    pub fn show_table(&self) -> String {
        let mut buf = String::new();

        for i in 0..5 {
            let card: String = self
                .community_cards
                .get(i)
                .map(|c| c.to_string())
                .unwrap_or("[    ]".to_string());
            buf.push_str(&card);
        }

        buf
    }

    #[inline]
    pub fn turn(&self) -> PlayerID {
        self.turn
    }

    #[inline]
    pub fn players(&self) -> &[Player] {
        &self.players
    }

    #[inline]
    pub fn community_cards(&self) -> &CardsDynamic {
        &self.community_cards
    }

    #[inline]
    pub fn deck(&self) -> &CardsDynamic {
        &self.deck
    }

    #[inline]
    pub fn state(&self) -> GameState {
        self.state
    }

    #[inline]
    pub fn dealer_position(&self) -> PlayerID {
        self.dealer
    }

    #[inline]
    pub fn current_player(&self) -> &Player {
        &self.players[self.turn]
    }

    #[inline]
    pub fn current_player_mut(&mut self) -> &mut Player {
        &mut self.players[self.turn]
    }

    pub fn pot(&self) -> Currency {
        debug_assert!(!self.players.is_empty());
        self.players.iter().map(|p| p.total_bet + p.round_bet).sum()
    }

    pub fn next_player(&self, turn: PlayerID) -> Option<PlayerID> {
        if self.players.len() < 2 {
            None
        } else {
            Some(if turn > self.players.len() - 1 {
                0
            } else {
                turn + 1
            })
        }
    }

    #[inline]
    fn active_players(&self) -> impl Iterator<Item = &Player> {
        self.players.iter().filter(|p| p.is_active())
    }

    pub fn next_active_player(&self, turn: PlayerID) -> Option<PlayerID> {
        let active: usize = self.active_players().count();
        if active < 2 {
            None
        } else {
            Some((turn + 1) % active)
        }
    }
}
