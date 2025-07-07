use std::fmt::Debug;

use rand::prelude::*;
use tracing::trace;

mod action;
pub mod cards;
pub mod evaluation;
mod phase;
mod state;
mod winner;

pub use action::*;
use cards::*;
use evaluation::*;
pub use phase::*;
pub use state::*;
pub use winner::*;

use crate::PoksError;
use crate::currency::Currency;
use crate::lobby::Seat;
use crate::players::{Player, PlayerID, PlayerState};
use crate::{CU, Result, err_int};

pub type GlogItem = (Option<PlayerID>, String);
pub type RNG = rand::rngs::StdRng;
pub type Seed = <RNG as rand::SeedableRng>::Seed;

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

// helper macros
macro_rules! current_player {
    ($self:tt) => {
        $self.players[$self.turn]
    };
}

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

    pub fn build(seats: &[Seat], dealer_pos: PlayerID) -> Result<Self> {
        let seed = Self::seed();
        Self::buid_with_seed(seats, dealer_pos, seed)
    }

    #[must_use]
    pub fn phase(&self) -> Phase {
        self.phase
    }

    #[must_use]
    pub fn phase_mut(&mut self) -> &mut Phase {
        &mut self.phase
    }

    pub fn set_phase(&mut self, phase: Phase) {
        for player in self.players.iter_mut() {
            player.total_bet += player.round_bet;
            player.round_bet = Currency::ZERO;
        }
        self.phase = phase;
        glogf!(self, None, "Phase: {phase}");
    }

    #[must_use]
    pub fn pot(&self) -> Currency {
        debug_assert!(!self.players.is_empty());
        self.players.iter().map(|p| p.total_bet + p.round_bet).sum()
    }

    #[must_use]
    pub fn highest_bet_of_round(&self) -> Currency {
        debug_assert!(!self.players.is_empty());
        self.players.iter().map(|p| p.round_bet).max().unwrap()
    }

    #[must_use]
    pub fn is_finished(&self) -> bool {
        self.winner.is_some()
    }

    pub fn set_winner(&mut self, w: Winner) {
        w.payout(self).expect("could not payout the winner");
        self.winner = Some(w);
        glog!(self, None, self.winner.unwrap().to_string())
    }

    #[must_use]
    pub fn winner(&self) -> Option<Winner> {
        self.winner
    }

    fn draw_card(&mut self) -> Card {
        self.deck.pop().unwrap()
    }

    #[inline]
    fn add_table_card(&mut self) {
        let c = self.draw_card();
        self.community_cards.push(c);
    }

    fn advance_phase(&mut self) {
        match self.phase() {
            Phase::Preflop => {
                let _ = self.draw_card(); // burn card
                for _ in 0..3 {
                    self.add_table_card();
                }
                assert_eq!(self.community_cards.len(), 3);
                self.set_phase(Phase::Flop);
            }
            Phase::Flop => {
                let _ = self.draw_card(); // burn card
                self.add_table_card();
                assert_eq!(self.community_cards.len(), 4);
                self.set_phase(Phase::Turn);
            }
            Phase::Turn => {
                let _ = self.draw_card(); // burn card
                self.add_table_card();
                assert_eq!(self.community_cards.len(), 5);
                self.set_phase(Phase::River);
                self.showdown();
            }
            Phase::River => unreachable!(),
        }
    }

    pub fn hand_plus_table(&self, pid: PlayerID) -> CardsDynamic {
        let player = &self.players[pid];
        let mut hand_plus_table: CardsDynamic = player.hand().into();
        hand_plus_table.extend(self.community_cards.iter());
        hand_plus_table.sort();
        hand_plus_table
    }

    fn showdown(&mut self) {
        let mut evals: Vec<(PlayerID, Eval<FiveCard>, Cards<7>)> = Vec::new();
        for (pid, player) in self.players.iter().enumerate() {
            if player.state != PlayerState::Playing {
                continue;
            }
            let mut hand_plus_table: CardsDynamic = player.hand().into();
            hand_plus_table.extend(self.community_cards.iter());
            hand_plus_table.sort();
            // TODO: add better result type and return this as error
            evals.push((
                pid,
                evaluator()
                    .evaluate_five(&*hand_plus_table)
                    .expect("could not evaluate"),
                hand_plus_table
                    .try_static()
                    .expect("Hands plus table were not 7 cards"),
            ));
        }

        evals.sort_by(|a, b| b.1.cmp(&a.1));
        if evals[0] == evals[1] {
            todo!("We have a draw!")
        }
        let winner = Winner::KnownCards(self.pot(), evals[0].0, evals[0].1, evals[0].2);
        self.set_winner(winner);
    }

    fn next_turn(&mut self) {
        self.turn = (self.turn + 1) % self.players.len();
        if self.turn == 0 {
            self.advance_phase();
        }
    }

    // BUG: this does not correctly do the betting rounds!
    pub fn process_action(&mut self, action: Option<Action>) -> Result<()> {
        let remaining_players = self.players.iter().filter(|p| p.state.is_playing()).count();
        if remaining_players == 1 {
            let winner_id = self
                .players
                .iter()
                .enumerate()
                .find(|(_, p)| p.state.is_playing())
                .map(|(id, _)| id)
                .ok_or_else(|| err_int!("No playing players found"))?;

            self.set_winner(Winner::UnknownCards(self.pot(), winner_id));
            return Ok(());
        }

        let round_bet = self.highest_bet_of_round();
        let player = &current_player!(self);

        if !player.state.is_playing() {
            self.next_turn();
        }

        let action = match action {
            Some(a) => a,
            None => return Ok(()), // come back with an action
        };

        if !current_player!(self).state.is_playing() {
            return Ok(()); // ignore
            // return Err(PoksError::player_not_playing(
            //     self.turn,
            //     format!("{:?}", current_player!(self).state),
            // ));
        }

        if current_player!(self).state == PlayerState::AllIn {
            self.next_turn();
            return Ok(());
        }
        match action {
            Action::Fold => {
                current_player!(self).state = PlayerState::Folded;
            }
            Action::Call(currency) => {
                if round_bet < current_player!(self).round_bet {
                    return Err(PoksError::InvalidCall);
                }
                let diff = round_bet - current_player!(self).round_bet;
                if diff != currency {
                    return Err(PoksError::call_mismatch(diff, currency));
                }
                if currency != CU!(0) {
                    current_player!(self).round_bet += currency;
                }
            }
            Action::Raise(currency) => {
                if self.state == GameState::RaiseDisallowed {
                    return Err(PoksError::RaiseNotAllowed);
                }
                current_player!(self).round_bet += currency;
            }
            Action::AllIn(currency) => {
                if current_player!(self).state == PlayerState::AllIn {
                    return Err(PoksError::PlayerAlreadyAllIn {
                        player_id: self.turn,
                    });
                }
                if self.state != GameState::RaiseDisallowed {
                    return Err(PoksError::RaiseNotAllowed);
                }
                current_player!(self).state = PlayerState::AllIn;
                current_player!(self).round_bet += currency;
            }
        }

        glogf!(self, self.turn, "{action}");

        self.next_turn();

        Ok(())
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

    pub fn turn(&self) -> PlayerID {
        self.turn
    }

    pub fn players(&self) -> &[Player] {
        &self.players
    }

    pub fn community_cards(&self) -> &CardsDynamic {
        &self.community_cards
    }

    pub fn deck(&self) -> &CardsDynamic {
        &self.deck
    }

    pub fn state(&self) -> GameState {
        self.state
    }

    pub fn action_call(&self) -> Action {
        let diff = self.highest_bet_of_round() - self.players[self.turn].round_bet;
        Action::Call(diff)
    }

    pub fn small_blind_position(&self) -> PlayerID {
        if self.players.len() == 2 {
            // In heads-up, dealer posts small blind
            self.dealer
        } else {
            (self.dealer + 1) % self.players.len()
        }
    }

    pub fn big_blind_position(&self) -> PlayerID {
        if self.players.len() == 2 {
            // In heads-up, non-dealer posts big blind
            (self.dealer + 1) % self.players.len()
        } else {
            (self.dealer + 2) % self.players.len()
        }
    }

    fn post_blinds(&mut self) -> Result<()> {
        let sb_pos = self.small_blind_position();
        let bb_pos = self.big_blind_position();

        let sbp = &mut self.players[sb_pos];
        *sbp.seat.behavior_mut().currency_mut() -= self.small_blind;
        sbp.round_bet += self.small_blind;
        glogf!(self, sb_pos, "Posts the small blind ({})", self.small_blind);

        let bbp = &mut self.players[bb_pos];
        *bbp.seat.behavior_mut().currency_mut() -= self.small_blind;
        self.players[bb_pos].round_bet += self.big_blind;
        glogf!(self, bb_pos, "Posts the big blind ({})", self.big_blind);

        Ok(())
    }

    pub fn gamelog(&self) -> &[GlogItem] {
        &self.game_log
    }

    pub fn take_gamelog(&mut self) -> Vec<GlogItem> {
        let a = self.game_log.clone();
        self.game_log = Vec::with_capacity(32);
        a
    }

    pub fn big_blind(&self) -> Currency {
        self.big_blind
    }

    pub fn small_blind(&self) -> Currency {
        self.small_blind
    }

    pub fn dealer_position(&self) -> PlayerID {
        self.dealer
    }
}
