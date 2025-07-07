use std::fmt::Debug;
use std::sync::OnceLock;

use poker::evaluate::FiveCardHandClass;
use poker::{Card, Eval, Evaluator, FiveCard, Suit};
use tracing::{debug, info};

use crate::currency::Currency;
use crate::errors::PoksError;
use crate::player::PlayerState;
use crate::transaction::Transaction;
use crate::world::AnyAccount;
use crate::{CU, Result, err_int};

mod impls; // additional trait impls

pub type PlayerID = usize;
pub type Cards<const N: usize> = [Card; N];
pub type GlogItem = (Option<PlayerID>, String);

pub static EVALUATOR: OnceLock<Evaluator> = OnceLock::new();

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
    UnknownCards(Currency, PlayerID),
    KnownCards(Currency, PlayerID, Eval<FiveCard>, Cards<7>),
}

#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct Player {
    state: PlayerState,
    total_bet: Currency,
    round_bet: Currency,
    hand: Cards<2>,
}

#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
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
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum Action {
    Fold,
    Call(Currency),
    Raise(Currency),
    AllIn(Currency),
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord, Default)]
#[non_exhaustive]
pub enum GameState {
    #[default]
    RaiseAllowed,
    RaiseDisallowed,
    Pause,
    Finished,
}

// helper macros
macro_rules! current_player {
    ($self:tt) => {
        $self.players[$self.turn]
    };
}

macro_rules! glog {
    ($self:tt, $player:expr, $($content:tt)+) => {
        $self.game_log.push((Some($player), format!($($content)+)))
    };
    ($self:tt, None ,$($content:tt)+) => {
        $self.game_log.push((None, format!($($content)+)))
    };
}

impl Game {
    pub fn build(
        player_amount: usize,
        accounts: &mut Vec<AnyAccount>,
        dealer_pos: PlayerID,
    ) -> Result<Self> {
        assert!(player_amount >= 2);
        let mut deck: CardsDynamic = poker::deck::shuffled().into();
        if player_amount > deck.len() / 2 {
            // TODO: return a proper error and result
            panic!("Not enough cards in a deck for this many players!")
        }
        let mut players = Vec::new();
        for _ in 0..player_amount {
            let hand: Cards<2> = [deck.pop().unwrap(), deck.pop().unwrap()];
            players.push(Player::new(hand));
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
        };

        game.post_blinds(accounts)?;

        Ok(game)
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
        self.winner = Some(w);
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
                let w = self.showdown();
            }
            Phase::River => unreachable!(),
        }
    }

    pub fn hand_plus_table(&self, pid: PlayerID) -> CardsDynamic {
        let player = &self.players[pid];
        let mut hand_plus_table: CardsDynamic = player.hand.into();
        hand_plus_table.extend(self.community_cards.iter());
        hand_plus_table.sort();
        hand_plus_table
    }

    fn showdown(&mut self) -> Result<Winner> {
        let mut evals: Vec<(PlayerID, Eval<FiveCard>, Cards<7>)> = Vec::new();
        for (pid, player) in self.players.iter().enumerate() {
            if player.state != PlayerState::Playing {
                continue;
            }
            let mut hand_plus_table: CardsDynamic = player.hand.into();
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

        Ok(winner)
    }

    fn next_turn(&mut self) {
        self.turn = (self.turn + 1) % self.players.len();
        if self.turn == 0 {
            self.advance_phase();
        }
    }

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
                    todo!("No betting allowed, just calling")
                }
                current_player!(self).state = PlayerState::AllIn;
                current_player!(self).round_bet += currency;
            }
        }

        self.next_turn();

        self.game_log.push((Some(self.turn), action.to_string()));

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

    fn post_blinds(&mut self, accounts: &mut Vec<AnyAccount>) -> Result<()> {
        let sb_pos = self.small_blind_position();
        let bb_pos = self.big_blind_position();

        assert_eq!(self.players.len(), accounts.len());

        *accounts[sb_pos].currency_mut() -= self.small_blind;
        self.players[sb_pos].round_bet += self.small_blind;
        glog!(self, sb_pos, "Posts the small blind ({})", self.small_blind);

        *accounts[bb_pos].currency_mut() -= self.big_blind;
        self.players[bb_pos].round_bet += self.big_blind;
        glog!(self, bb_pos, "Posts the big blind ({})", self.big_blind);

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
}

impl Player {
    #[must_use]
    #[inline]
    pub fn show_hand(&self) -> String {
        show_cards(&self.hand)
    }

    pub fn new(hand: Cards<2>) -> Self {
        Self {
            state: Default::default(),
            total_bet: Default::default(),
            round_bet: Default::default(),
            hand,
        }
    }

    pub fn hand(&self) -> [Card; 2] {
        self.hand
    }

    pub fn state(&self) -> PlayerState {
        self.state
    }

    pub fn total_bet(&self) -> Currency {
        self.total_bet
    }

    pub fn round_bet(&self) -> Currency {
        self.round_bet
    }
}

impl GameState {
    #[inline]
    #[must_use]
    pub fn is_ongoing(&self) -> bool {
        match self {
            GameState::RaiseAllowed | GameState::RaiseDisallowed => true,
            GameState::Pause | GameState::Finished => false,
        }
    }
}

impl Action {
    pub fn prepare_transaction(&self) -> Option<Transaction> {
        match self {
            Action::Call(currency) | Action::Raise(currency) | Action::AllIn(currency) => {
                Some(Transaction::new(*currency))
            }
            Action::Fold => None,
        }
    }
    #[inline]
    pub fn check() -> Self {
        Self::Call(CU!(0))
    }
}

impl Winner {
    pub fn payout(&self, game: &Game, player: &mut AnyAccount) -> Result<()> {
        info!("Payout!");
        let old = *player.currency();
        let winnings = game.pot();
        assert_ne!(winnings, CU!(0));
        *player.currency_mut() += game.pot();
        assert_eq!(old + winnings, *player.currency());
        debug!("After Payout? {}", player.currency());
        Ok(())
    }

    pub fn pid(&self) -> PlayerID {
        match self {
            Winner::UnknownCards(_, pid) => *pid,
            Winner::KnownCards(_, pid, ..) => *pid,
        }
    }
}

pub fn show_cards(cards: &[Card]) -> String {
    let mut buf = String::new();
    for card in cards {
        buf.push_str(&card.to_string());
    }
    buf
}

#[inline]
pub fn evaluator() -> &'static Evaluator {
    EVALUATOR.get_or_init(Evaluator::new)
}

pub fn show_eval_cards(cls: FiveCardHandClass, cards: &Cards<7>) -> String {
    assert!(cards.is_sorted());

    let todo = String::from("todo");
    macro_rules! scards {
        ($collection:expr) => {{
            $collection.sort();
            $collection.reverse();
            $collection.into_iter().map(|c| c.to_string()).collect()
        }};
    }
    macro_rules! filter {
        ($cards:tt, $filter:expr) => {{
            let mut _v: Vec<_> = $cards.iter().rev().filter($filter).collect();
            _v
        }};
    }
    macro_rules! fcards {
        ($filter:expr) => {
            scards!(filter!(cards, $filter))
        };
    }
    macro_rules! flush {
        ($cards:tt) => {{
            let mut v: [Vec<&Card>; 4] = [
                filter!($cards, |c| c.suit() == Suit::Clubs),
                filter!($cards, |c| c.suit() == Suit::Hearts),
                filter!($cards, |c| c.suit() == Suit::Spades),
                filter!($cards, |c| c.suit() == Suit::Diamonds),
            ];
            v.sort_by_key(|b| std::cmp::Reverse(b.len()));
            let longest = &mut v[0];
            longest.truncate(5);
            longest.clone()
        }};
    }
    match cls {
        FiveCardHandClass::HighCard { .. } => cards[6].to_string(),
        FiveCardHandClass::Pair { rank } => fcards!(|c| c.rank() == rank),
        FiveCardHandClass::TwoPair {
            high_rank,
            low_rank,
        } => fcards!(|c| c.rank() == high_rank || c.rank() == low_rank),
        FiveCardHandClass::ThreeOfAKind { rank } => fcards!(|c| c.rank() == rank),
        FiveCardHandClass::Straight { rank } => {
            let mut v = filter!(cards, |c| c.rank() <= rank);
            v.truncate(5);
            v.reverse();
            scards!(v)
        }
        FiveCardHandClass::Flush { .. } => scards!(flush!(cards)),
        FiveCardHandClass::FullHouse { trips, pair } => {
            fcards!(|c| c.rank() == pair || c.rank() == trips)
        }
        FiveCardHandClass::FourOfAKind { rank } => fcards!(|c| c.rank() == rank),
        FiveCardHandClass::StraightFlush { rank } => {
            let flushcards: Vec<&Card> = flush!(cards);
            let mut v = filter!(flushcards, |c| c.rank() <= rank);
            v.truncate(5);
            v.reverse();
            scards!(v)
        }
    }
    // show_cards(&cards)
}

#[cfg(test)]
mod test {
    use poker::{Card, cards};

    use crate::{
        game::{evaluator, show_eval_cards},
        len_to_const_arr,
    };

    #[test]
    fn test_show_eval_cards() {
        let r: Vec<(Vec<_>, &str)> = vec![
            (cards!("Th 2c 3c 4c 5c 7h 8h").collect(), "[ T♥ ]"), // high card
            (cards!("Th Tc 3c 4c 5c 7h 8h").collect(), "[ T♣ ][ T♥ ]"), // pair
            (
                cards!("Th Tc 3c 3h 5c 7h 8h").collect(),
                "[ T♣ ][ T♥ ][ 3♥ ][ 3♣ ]",
            ), // two pair
            (
                cards!("Th Tc Td 5c 6h 7h 8h").collect(),
                "[ T♦ ][ T♣ ][ T♥ ]",
            ), // set
            (
                cards!("Th 3c 4c 5c 6h 7h 8h").collect(),
                "[ 8♥ ][ 7♥ ][ 6♥ ][ 5♣ ][ 4♣ ]",
            ), // straight
            (
                cards!("Th 3h 4h 5c 6h 7h 8h").collect(),
                "[ T♥ ][ 8♥ ][ 7♥ ][ 6♥ ][ 4♥ ]",
            ), // flush
            (
                cards!("Th Tc Td 5c 5h 7h 8h").collect(),
                "[ T♦ ][ T♣ ][ T♥ ][ 5♥ ][ 5♣ ]",
            ), // full house
            (
                cards!("Th Tc Td Ts 6h 7h 8h").collect(),
                "[ T♠ ][ T♦ ][ T♣ ][ T♥ ]",
            ), // quads
            (
                cards!("9h 3c 4h 5h 6h 7h 8h").collect(),
                "[ 9♥ ][ 8♥ ][ 7♥ ][ 6♥ ][ 5♥ ]",
            ), // straight flush
        ];
        for (cards, show) in r {
            let mut cards: Vec<Card> = cards.into_iter().map(|a| a.unwrap()).collect();
            cards.sort();
            let cards = len_to_const_arr(&cards).unwrap();
            assert_eq!(
                show_eval_cards(evaluator().evaluate_five(cards).unwrap().classify(), &cards),
                show
            );
        }
    }
}
