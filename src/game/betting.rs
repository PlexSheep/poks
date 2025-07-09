use super::*;
use crate::Result;

/// Implement the blinds
impl super::Game {
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

    pub(crate) fn post_blinds(&mut self) -> Result<()> {
        let sb_pos = self.small_blind_position();
        let bb_pos = self.big_blind_position();

        let sbp = &mut self.players[sb_pos];
        *sbp.currency_mut() -= self.small_blind;
        sbp.round_bet += self.small_blind;
        glogf!(self, sb_pos, "Posts the small blind ({})", self.small_blind);

        let bbp = &mut self.players[bb_pos];
        *bbp.currency_mut() -= self.small_blind;
        self.players[bb_pos].round_bet += self.big_blind;
        glogf!(self, bb_pos, "Posts the big blind ({})", self.big_blind);

        Ok(())
    }

    pub fn big_blind(&self) -> Currency {
        self.big_blind
    }

    pub fn small_blind(&self) -> Currency {
        self.small_blind
    }
}

/// Implement the betting rounds and game phases
impl super::Game {
    pub fn highest_bet_of_round(&self) -> Currency {
        debug_assert!(!self.players.is_empty());
        self.players.iter().map(|p| p.round_bet).max().unwrap()
    }

    pub(super) fn advance_turn(&mut self) -> Result<()> {
        todo!()
    }

    pub(super) fn advance_phase(&mut self) -> Result<()> {
        match self.phase() {
            Phase::Preflop => {
                let _ = self.draw_card(); // burn card
                for _ in 0..3 {
                    self.add_table_card();
                }
                assert_eq!(self.community_cards.len(), 3);
                self.set_phase(Phase::Flop);
                self.start_betting()
            }
            Phase::Flop => {
                let _ = self.draw_card(); // burn card
                self.add_table_card();
                assert_eq!(self.community_cards.len(), 4);
                self.set_phase(Phase::Turn);
                self.start_betting()
            }
            Phase::Turn => {
                let _ = self.draw_card(); // burn card
                self.add_table_card();
                assert_eq!(self.community_cards.len(), 5);
                self.set_phase(Phase::River);
                self.start_betting()
            }
            Phase::River => self.showdown(),
        }
    }

    fn start_betting(&mut self) -> Result<()> {
        todo!()
    }

    fn showdown(&mut self) -> Result<()> {
        let mut evals: Vec<(PlayerID, Eval<FiveCard>, Cards<7>)> = Vec::new();
        for (pid, player) in self.players.iter().enumerate() {
            if player.state != PlayerState::Playing {
                continue;
            }
            let mut hand_plus_table: CardsDynamic = player.hand().into();
            hand_plus_table.extend(self.community_cards.iter());
            hand_plus_table.sort();
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
        if evals.iter().any(|e| e.1 == evals[0].1) {
            todo!("We have a draw!")
        }
        let winner = Winner::KnownCards(self.pot(), evals[0].0, evals[0].1, evals[0].2);
        self.set_winner(winner);
        Ok(())
    }

    pub(super) fn is_betting_complete(&self) -> bool {
        todo!()
    }
}

/// Helper functions for players trying to decide on an [`Action`]
impl super::Game {
    /// Helper: Can the current player check?
    pub fn can_check(&self, player: PlayerID) -> bool {
        let player = &self.players[player];
        player.round_bet == self.highest_bet_of_round()
    }

    /// Helper: Get minimum raise amount
    // TODO: This isn't actually correct. In holdem, you must raise at least as much as the last raise,
    // I think?
    pub fn min_raise_amount(&self) -> Currency {
        self.big_blind
    }
}
