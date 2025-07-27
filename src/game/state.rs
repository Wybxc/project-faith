use std::time::Duration;

use crate::{
    game::{
        card::{Card, CardId, REGISTRY},
        player::PlayerState,
    },
    grpc,
    utils::Timer,
};

/// 游戏状态
///
/// 可变性限制：public API 只允许通过 `Action` trait 来修改状态，确保状态变更的可控性。
#[derive(Default)]
pub struct GameState {
    players: (PlayerState, PlayerState),

    /// The current round number.
    round: u32,

    /// Indicates if the game is finished.
    finished: bool,

    /// Current player's turn.
    current_turn: PlayerId,

    /// Timer for the current turn.
    turn_timer: Timer,

    /// Debug log for tracing actions.
    debug_log: Vec<String>,
}

impl GameState {
    pub fn new() -> Self {
        Default::default()
    }

    fn initialize(
        &mut self,
        player0_deck: Vec<CardId>,
        player1_deck: Vec<CardId>,
        player0_faith: Vec<CardId>,
        player1_faith: Vec<CardId>,
    ) {
        self.players.0.initialize(player0_deck, player0_faith);
        self.players.1.initialize(player1_deck, player1_faith);
        self.round = 0;
        self.finished = false;
        self.current_turn = PlayerId::Player0;
        self.debug_log.clear();
    }

    pub fn me(&self, player: PlayerId) -> &PlayerState {
        match player {
            PlayerId::Player0 => &self.players.0,
            PlayerId::Player1 => &self.players.1,
        }
    }

    fn me_mut(&mut self, player: PlayerId) -> &mut PlayerState {
        match player {
            PlayerId::Player0 => &mut self.players.0,
            PlayerId::Player1 => &mut self.players.1,
        }
    }

    pub fn turn_time_remaining(&self) -> Duration {
        self.turn_timer.remaining()
    }

    pub fn to_client(&self, player: PlayerId) -> grpc::GameState {
        let debug_log = self.debug_log.clone();
        let self_hand = self.me(player).hand.iter().map(|id| id.0).collect();
        let other_hand_count = self.me(player.opp()).hand.len() as u32;
        let self_deck_count = self.me(player).deck.len() as u32;
        let other_deck_count = self.me(player.opp()).deck.len() as u32;
        let round_number = self.round;
        let is_my_turn = self.current_turn == player;
        let game_finished = self.finished;
        let self_faith_cards = self.me(player).faith.iter().map(|id| id.0).collect();
        let other_faith_cards = self.me(player.opp()).faith.iter().map(|id| id.0).collect();
        grpc::GameState {
            debug_log,
            self_hand,
            other_hand_count,
            self_deck_count,
            other_deck_count,
            round_number,
            is_my_turn,
            game_finished,
            self_faith_cards,
            other_faith_cards,
        }
    }

    /// Performs an action on the game state.
    pub fn perform<A: Action>(&mut self, action: A) -> A::Output {
        let output = action.perform(self);
        self.debug_log.push(action.debug_log());
        output
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum PlayerId {
    #[default]
    Player0 = 0,
    Player1 = 1,
}

impl PlayerId {
    /// Returns the opposite player ID.
    pub fn opp(self) -> PlayerId {
        match self {
            PlayerId::Player0 => PlayerId::Player1,
            PlayerId::Player1 => PlayerId::Player0,
        }
    }
}

pub trait Action {
    type Output;

    fn perform(&self, game_state: &mut GameState) -> Self::Output;
    fn debug_log(&self) -> String;
}

/// 初始化游戏状态
pub struct Initalize;

impl Action for Initalize {
    type Output = ();

    fn perform(&self, game_state: &mut GameState) {
        game_state.initialize(
            vec![CardId(7001); 30], // Player 0's deck
            vec![CardId(7002); 30], // Player 1's deck
            vec![CardId(8001); 3],  // Player 0's faith
            vec![CardId(8001); 3],  // Player 1's faith
        );
    }

    fn debug_log(&self) -> String {
        "游戏开始。".to_string()
    }
}

/// 回合开始
pub struct TurnStart {
    pub player: PlayerId,
}

impl Action for TurnStart {
    type Output = ();

    fn perform(&self, game_state: &mut GameState) {
        game_state.current_turn = self.player;
        game_state.turn_timer.reset(Duration::from_secs(30));
        game_state.turn_timer.start();
    }

    fn debug_log(&self) -> String {
        format!("回合开始，当前玩家：{}", self.player as u8)
    }
}

/// 玩家抽牌
pub struct DrawCards {
    pub player: PlayerId,
    pub count: usize,
}

impl Action for DrawCards {
    type Output = Vec<CardId>;

    fn perform(&self, game_state: &mut GameState) -> Self::Output {
        let player_state = game_state.me_mut(self.player);
        let mut drawn_cards = Vec::new();
        for _ in 0..self.count {
            if let Some(card) = player_state.deck.pop() {
                drawn_cards.push(card);
                player_state.hand.push(card);
            }
        }
        drawn_cards
    }

    fn debug_log(&self) -> String {
        format!("玩家 {} 抽了 {} 张牌。", self.player as u8, self.count)
    }
}

/// 玩家出牌（开始）
pub struct PlayCard {
    pub player: PlayerId,
    pub card_index: usize,
}

impl Action for PlayCard {
    type Output = Option<CardId>;

    fn perform(&self, game_state: &mut GameState) -> Self::Output {
        let player_state = game_state.me_mut(self.player);
        if let Some(card) = player_state.hand.get(self.card_index).cloned() {
            player_state.hand.remove(self.card_index);
            Some(card)
        } else {
            None
        }
    }

    fn debug_log(&self) -> String {
        format!(
            "玩家 {} 使用了第 {} 张手牌。",
            self.player as u8, self.card_index
        )
    }
}

/// 执行卡牌效果
pub struct ExecuteCard {
    pub player: PlayerId,
    pub card_id: CardId,
}

impl Action for ExecuteCard {
    type Output = ();

    fn perform(&self, game_state: &mut GameState) {
        let Some(card) = REGISTRY.cards.get(&self.card_id) else {
            return; // 卡牌不存在
        };
        match card {
            Card::Order(order_card) => {
                for skill in &order_card.skills {
                    skill(game_state, self.player);
                }
            }
            Card::Faith(_) => {}
        }
    }

    fn debug_log(&self) -> String {
        format!(
            "玩家 {} 执行了卡牌编号 {} 的效果。",
            self.player as u8, self.card_id.0
        )
    }
}

pub struct EndTurn {
    pub player: PlayerId,
}

impl Action for EndTurn {
    type Output = ();

    fn perform(&self, game_state: &mut GameState) {
        game_state.turn_timer.pause();
    }

    fn debug_log(&self) -> String {
        format!("玩家 {} 结束了回合。", self.player as u8)
    }
}

/// 回合结束，增加回合数
pub struct BumpRound;

impl Action for BumpRound {
    type Output = ();

    fn perform(&self, game_state: &mut GameState) {
        game_state.round += 1;
    }

    fn debug_log(&self) -> String {
        "回合数增加。".to_string()
    }
}

/// 游戏结束
pub struct GameFinished;

impl Action for GameFinished {
    type Output = ();

    fn perform(&self, game_state: &mut GameState) {
        game_state.finished = true;
    }

    fn debug_log(&self) -> String {
        "游戏结束。".to_string()
    }
}
