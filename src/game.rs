use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use std::{fmt::Debug, sync::RwLock};

use circular_queue::CircularQueue;
use poker::{Card, Evaluator};
use tracing::{debug, info, trace, warn};

mod impls; // additional trait impls

pub type Currency = u64;
pub type Result<T> = color_eyre::Result<T>;

pub static LOCAL_USER_ACTION_READY: AtomicBool = AtomicBool::new(false);
pub static LOCAL_USER_ACTION: RwLock<Action> = RwLock::new(Action::Check);
pub const ACTION_LOG_SIZE: usize = 2000;

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct Hand(Card, Card);

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord, Default)]
pub enum Phase {
    #[default]
    Preflop,
    Flop,
    Turn,
    River,
}

#[derive(Clone, PartialEq, Eq)]
pub struct World {
    evaluator: Arc<Evaluator>,
    pub players: Vec<Player>,
    pub game: Game,
    deck: Vec<Card>,
    action_log: CircularQueue<(Option<usize>, Action)>,
}

#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct Game {
    pub phase: Phase,
    pub turn: usize,
    pub player_states: Vec<PlayerState>,
    pub player_total_bets: Vec<Currency>,
    pub table_cards: Vec<Card>,
}

#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum Player {
    Local(PlayerLocal),
    CPU(PlayerCPU),
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum Action {
    HiddenWait,
    Fold,
    Check,
    Raise(Currency),
    AllIn,
    NewGame,
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum PlayerState {
    Playing,
    Folded,
    Paused,
    Lost,
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum GameState {
    CPUPlayerDidSomething,
    Pause,
    AwaitingLocalPlayer,
}

pub trait PlayerBehavior {
    fn hand(&self) -> &Option<Hand>;
    fn hand_mut(&mut self) -> &mut Option<Hand>;
    fn currency(&self) -> &Currency;
    fn currency_mut(&mut self) -> &mut Currency;
    fn act(&self, game: &Game) -> Action;

    fn set_hand(&mut self, new: Hand) {
        *self.hand_mut() = Some(new);
    }
    fn set_currency(&mut self, new: Currency) {
        *self.currency_mut() = new;
    }
    fn win(&mut self, game: Game) {
        *self.currency_mut() += game.pot();
    }
}

#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord, Default)]
pub struct PlayerLocal {
    hand: Option<Hand>,
    currency: Currency,
}

#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord, Default)]
pub struct PlayerCPU {
    hand: Option<Hand>,
    currency: Currency,
}

macro_rules! call_enum_functions {
    ($self:expr,$($function:tt)+) => {
        match $self {
            Player::Local(p) => p.$($function)+,
            Player::CPU(p) => p.$($function)+,
        }
    };
}

impl PlayerBehavior for Player {
    fn hand(&self) -> &Option<Hand> {
        call_enum_functions!(self, hand())
    }

    fn hand_mut(&mut self) -> &mut Option<Hand> {
        call_enum_functions!(self, hand_mut())
    }

    fn currency(&self) -> &Currency {
        call_enum_functions!(self, currency())
    }

    fn currency_mut(&mut self) -> &mut Currency {
        call_enum_functions!(self, currency_mut())
    }
    fn act(&self, game: &Game) -> Action {
        call_enum_functions!(self, act(game))
    }
}

macro_rules! player_impl {
    ($struct:ident, $($extra:tt)+) => {
        impl PlayerBehavior for $struct {
            fn hand(&self) -> &Option<Hand> {
                &self.hand
            }

            fn hand_mut(&mut self) -> &mut Option<Hand> {
                &mut self.hand
            }
            fn currency(&self) -> &Currency {
                &self.currency
            }
            fn currency_mut(&mut self) -> &mut Currency {
                &mut self.currency
            }
            $($extra)+
        }
    };
}

impl PlayerLocal {
    pub fn set_action_is_ready(ready: bool) {
        LOCAL_USER_ACTION_READY.store(ready, std::sync::atomic::Ordering::Relaxed);
    }
    pub fn get_action_is_ready() -> bool {
        LOCAL_USER_ACTION_READY.load(std::sync::atomic::Ordering::Relaxed)
    }
    pub fn set_action(action: Action) {
        *LOCAL_USER_ACTION
            .write()
            .expect("could not read local user action") = action;
    }
    pub fn get_action() -> Action {
        assert!(Self::get_action_is_ready());
        *LOCAL_USER_ACTION
            .read()
            .expect("could not read local user action")
    }
}

player_impl!(
    PlayerLocal,
    fn act(&self, _game: &Game) -> Action {
        // HACK: this is horrible from design, I should have some way to pass an argument to this
        // from the ui!

        if !Self::get_action_is_ready() {
            return Action::HiddenWait;
        }
        let a = Self::get_action();
        Self::set_action_is_ready(false);
        a
    }
);
player_impl!(
    PlayerCPU,
    fn act(&self, _game: &Game) -> Action {
        let a = rand::random();
        match a {
            Action::Raise(bet) => {
                if self.currency < bet {
                    return Action::Check;
                }
                a
            }
            a => a,
        }
    }
);

impl Player {
    pub const fn local(currency: Currency) -> Self {
        Self::Local(PlayerLocal {
            hand: None,
            currency,
        })
    }
    pub const fn cpu(currency: Currency) -> Self {
        Self::CPU(PlayerCPU {
            hand: None,
            currency,
        })
    }
}

impl World {
    pub fn new(players_amount: usize) -> Self {
        let evaluator = Evaluator::new().into();
        let mut players = vec![Player::local(5000)];
        for _ in 1..players_amount {
            players.push(Player::cpu(5000))
        }
        let deck = poker::deck::shuffled();
        debug_assert_eq!(deck.len(), 52);
        let mut w = Self {
            evaluator,
            game: Game::new(players.len()), // dummy
            players,
            deck,
            action_log: CircularQueue::with_capacity(ACTION_LOG_SIZE),
        };
        w.start_new_game();
        for player in &w.players {
            assert!(player.hand().is_some())
        }
        w
    }

    pub fn shuffle_cards(&mut self) {
        self.deck = poker::deck::shuffled();
        debug_assert_eq!(self.deck.len(), 52)
    }

    #[must_use]
    pub fn draw_card(&mut self) -> Card {
        self.deck.pop().expect("the deck was empty!")
    }

    pub fn start_new_game(&mut self) {
        self.shuffle_cards();
        let game = Game::new(self.players.len());

        for pi in 0..self.players.len() {
            let hand: Hand = [self.draw_card(), self.draw_card()].into();
            let player = &mut self.players[pi];
            player.set_hand(hand);
        }

        self.game = game;
    }

    pub fn tick_game(&mut self) -> Result<GameState> {
        debug_assert!(self.game.turn < self.players.len());
        let player_action = self.players[self.game.turn].act(&self.game);
        self.process_player_action(player_action)?;
        Ok(
            if matches!(self.players[self.game.turn], Player::Local(_)) {
                GameState::AwaitingLocalPlayer
            } else {
                GameState::CPUPlayerDidSomething
            },
        )
    }

    fn process_player_action(&mut self, action: Action) -> Result<()> {
        let player = &mut self.players[self.game.turn];
        let current_state = self.game.player_states[self.game.turn];
        if current_state != PlayerState::Playing {
            info!(
                "Player cannot do anything because they are {}",
                current_state
            );
            self.game.turn += 1;
            self.game.turn %= self.players.len();
            return Ok(());
        }
        match action {
            Action::Fold => {
                self.game.player_states[self.game.turn] = PlayerState::Folded;
            }
            Action::Raise(bet) => {
                debug_assert!(*player.currency() >= bet);
                *player.currency_mut() -= bet;
                self.game.player_total_bets[self.game.turn] += bet;
            }
            Action::AllIn => {
                self.game.player_total_bets[self.game.turn] += player.currency();
                player.set_currency(0);
            }
            Action::Check => {
                let highest_bet = self.game.highest_bet();
                let player_total = self.game.player_total_bets[self.game.turn];
                if player_total < highest_bet {
                    let diff = highest_bet - player_total;
                    if *player.currency() < diff {
                        // player goes all in
                        return self.process_player_action(Action::AllIn);
                    } else {
                        *player.currency_mut() -= diff;
                        self.game.player_total_bets[self.game.turn] += diff;
                        assert_eq!(self.game.player_total_bets[self.game.turn], highest_bet);
                    }
                }
            }
            Action::HiddenWait => {
                return Ok(());
            }
            _ => {
                self.action_log.push((None, action));
                return Ok(());
            }
        }
        self.action_log.push((Some(self.game.turn), action));
        self.game.turn += 1;
        if self.game.turn >= self.players.len() {
            self.advance_phase()?;
        }
        Ok(())
    }

    pub fn show_table(&self) -> String {
        let mut buf = String::new();

        for i in 0..5 {
            let card: String = self
                .game
                .table_cards
                .get(i)
                .map(|c| c.to_string())
                .unwrap_or("[    ]".to_string());
            buf.push_str(&card);
        }

        buf
    }

    fn add_table_card(&mut self) {
        let card = self.draw_card();
        self.game.table_cards.push(card);
    }

    fn advance_phase(&mut self) -> Result<()> {
        self.game.turn = 0;
        if !self.bets_complete() {
            return Ok(());
        };
        match self.game.phase() {
            Phase::Preflop => {
                let _ = self.draw_card(); // burn card
                for _ in 0..3 {
                    self.add_table_card();
                }
                assert_eq!(self.game.table_cards.len(), 3);
                self.game.set_phase(Phase::Flop);
            }
            Phase::Flop => {
                let _ = self.draw_card(); // burn card
                self.add_table_card();
                assert_eq!(self.game.table_cards.len(), 4);
                self.game.set_phase(Phase::Turn);
            }
            Phase::Turn => {
                let _ = self.draw_card(); // burn card
                self.add_table_card();
                assert_eq!(self.game.table_cards.len(), 5);
                self.game.set_phase(Phase::River);
            }
            Phase::River => {
                todo!("SHOWDOWN")
            }
        }
        Ok(())
    }

    fn bets_complete(&mut self) -> bool {
        let highest_bet = self.game.highest_bet();
        if self
            .players
            .iter()
            .enumerate()
            .all(|(pi, _)| self.game.player_total_bets[pi] == highest_bet)
        {
            assert!(
                self.players
                    .iter()
                    .enumerate()
                    .all(|(pi, _)| self.game.player_total_bets[pi] == highest_bet)
            );
            true
        } else {
            info!("highest bet is {}", self.game.highest_bet());
            info!("Bets are not done!");
            false
        }
    }

    pub fn action_log(&self) -> &CircularQueue<(Option<usize>, Action)> {
        &self.action_log
    }
}

impl Game {
    pub fn new(player_amount: usize) -> Self {
        Game {
            turn: 0,
            phase: Phase::default(),
            player_states: vec![PlayerState::Playing; player_amount],
            player_total_bets: vec![0; player_amount],
            table_cards: Vec::with_capacity(5),
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
}

pub fn show_hand(h: Option<Hand>) -> String {
    h.map(|h| h.to_string()).unwrap_or("(No Hand)".to_string())
}
