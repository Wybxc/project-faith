use crate::{
    game::{
        card::{Card, CardId, REGISTRY},
        player::PlayerState,
    },
    grpc,
};

/// 游戏状态
///
/// 可变性限制：public API 只允许通过 `Action` trait 来修改状态，确保状态变更的可控性。
pub struct GameState {
    players: (PlayerState, PlayerState),

    /// The current round number.
    round: u32,
}

impl GameState {
    pub fn new() -> Self {
        let p0 = PlayerState::new();
        let p1 = PlayerState::new();
        Self {
            players: (p0, p1),
            round: 0,
        }
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

    pub fn to_client(&self, player: PlayerId) -> grpc::GameState {
        let self_hand = self.me(player).hand.iter().map(|id| id.0).collect();
        let other_hand_count = self.me(player.opp()).hand.len() as u32;
        let self_deck_count = self.me(player).deck.len() as u32;
        let other_deck_count = self.me(player.opp()).deck.len() as u32;
        let round_number = self.round;

        grpc::GameState {
            self_hand,
            other_hand_count,
            self_deck_count,
            other_deck_count,
            round_number,
        }
    }

    /// Performs an action on the game state.
    pub fn perform<A: Action>(&mut self, action: A) -> A::Output {
        action.perform(self)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlayerId {
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
}

/// 初始化游戏状态
pub struct Initalize;

impl Action for Initalize {
    type Output = ();

    fn perform(&self, game_state: &mut GameState) {
        game_state.players.0.initialize(vec![CardId(7001); 30]);
        game_state.players.1.initialize(vec![CardId(7002); 30]);
        game_state.round = 0;
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
        }
    }
}

/// 回合结束，增加回合数
pub struct BumpRound;

impl Action for BumpRound {
    type Output = ();

    fn perform(&self, game_state: &mut GameState) {
        game_state.round += 1;
    }
}
