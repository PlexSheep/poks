use std::fmt::Debug;

use poker::{Card, Eval, Evaluator, FiveCard};

use crate::player::{PlayerBehavior, PlayerState};
use crate::{Result, len_to_const_arr};

mod impls; // additional trait impls

pub type PlayerID = usize;
pub type Cards<const N: usize> = [Card; N];

#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord, Default)]
pub struct CardsDynamic {
    inner: Vec<Card>,
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord, Default)]
pub enum Phase {
    #[default]
    Preflop,
    Flop,
    Turn,
    River,
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum Winner {
    UnknownCards(PlayerID),
    KnownCards(PlayerID, Eval<FiveCard>, Cards<7>),
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct Player {
    state: PlayerState,
    total_bet: Currency,
    round_bet: Currency,
    hand: Cards<2>,
}

#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct Game {
    pub phase: Phase,
    pub turn: PlayerID,
    pub players: Vec<Player>,
    pub community_cards: CardsDynamic,
    winner: Option<Winner>,
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum Action {
    Fold,
    Call,
    Check,
    Raise(Currency),
    AllIn,
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
#[non_exhaustive]
pub enum GameState {
    Ongoing,
    Pause,
    Finished,
}

impl Game {
    pub fn new(player_amount: usize) -> Self {
        Game {
            turn: 0,
            phase: Phase::default(),
            players,
            community_cards: [],
            winner: None,
        }
    }

    pub fn phase(&self) -> Phase {
        self.phase
    }

    pub fn phase_mut(&mut self) -> &mut Phase {
        &mut self.phase
    }

    pub fn set_phase(&mut self, phase: Phase) {
        self.phase = phase;
    }

    pub fn pot(&self) -> Currency {
        self.player_total_bets.iter().sum()
    }

    pub fn highest_bet(&self) -> Currency {
        assert!(!self.player_total_bets.is_empty());
        *self.player_total_bets.iter().max().unwrap()
    }

    pub fn is_finished(&self) -> bool {
        self.winner.is_some()
    }

    pub fn set_winner(&mut self, w: Winner) {
        self.winner = Some(w);
    }

    pub fn winner(&self) -> Option<Winner> {
        self.winner
    }

    pub fn add_table_card(&mut self, card: Card) {
        self.community_cards.push(card);
    }

    pub fn advance_phase<F: FnOnce() -> Card>(&mut self, draw_card: F) {
        match self.phase() {
            Phase::Preflop => {
                let _ = draw_card(); // burn card
                for _ in 0..3 {
                    self.add_table_card(draw_card());
                }
                assert_eq!(self.community_cards.len(), 3);
                self.set_phase(Phase::Flop);
            }
            Phase::Flop => {
                let _ = draw_card(); // burn card
                self.add_table_card(draw_card());
                assert_eq!(self.community_cards.len(), 4);
                self.set_phase(Phase::Turn);
            }
            Phase::Turn => {
                let _ = draw_card(); // burn card
                self.add_table_card(draw_card());
                assert_eq!(self.community_cards.len(), 5);
                self.set_phase(Phase::River);
                let w = self.showdown()?;
                self.action_log.push((None, Action::Winner(w)));
            }
            Phase::River => unreachable!(),
        }
    }

    pub fn showdown(&mut self) -> Result<Winner> {
        let mut evals = Vec::new();
        for (pid, player) in self.players.iter().enumerate() {
            if player.state != PlayerState::Playing {
                continue;
            }
            let mut hand_plus_table: CardsDynamic = player.hand.into();
            hand_plus_table.extend(&self.community_cards);
            // TODO: add better result type and return this as error
            evals.push((
                pid,
                self.evaluator
                    .evaluate_five(&hand_plus_table)
                    .expect("could not evaluate"),
                len_to_const_arr(&hand_plus_table)?,
            ));
        }

        evals.sort_by(|a, b| a.1.cmp(&b.1));
        let winner = Winner::KnownCards(evals[0].0, evals[0].1, evals[0].2);
        self.game.set_winner(winner);

        Ok(winner)
    }

    pub fn process_action(&self, action: Action) -> Result<GameState> {
        todo!()
    }
}

pub fn show_hand(h: Option<Hand>) -> String {
    h.map(|h| h.to_string()).unwrap_or("(No Hand)".to_string())
}

pub fn show_cards(cards: &[Card]) -> String {
    let mut buf = String::new();
    for card in cards {
        buf.push_str(&card.to_string());
    }
    buf
}
