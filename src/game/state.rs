use std::{time::Duration, vec};

use crate::{
    game::{
        card::{CardDef, CardId, InDeck, InHand, REGISTRY},
        player::{CurrentTurn, PlayerId, PlayerState},
    },
    system::{Entity, System, exact},
    utils::Timer,
};

/// 游戏状态
///
/// 可变性限制：public API 只允许通过 `Action` trait 来修改状态，确保状态变更的可控性。
pub struct GameState {
    /// The current round number.
    pub round: u32,

    /// Indicates if the game is finished.
    pub finished: bool,
}

impl GameState {
    pub fn new() -> Self {
        Self {
            round: 0,
            finished: false,
        }
    }
}

pub struct TurnTimer(pub Timer);

#[derive(Default, Clone)]
pub struct DebugLog {
    pub entries: Vec<String>,
}

impl DebugLog {
    pub fn push(&mut self, entry: impl Into<String>) {
        self.entries.push(entry.into());
    }
}

pub trait Action {
    type Output;

    fn perform(&self, system: &mut System) -> Self::Output;
}

/// 初始化游戏状态
pub struct Initalize;

impl Action for Initalize {
    type Output = ();

    fn perform(&self, system: &mut System) {
        system.add_resource(GameState::new());

        system
            .entity()
            .component(PlayerId::Player0)
            .component_with(|system| {
                PlayerState::new(
                    system,
                    PlayerId::Player0,
                    vec![CardId(7001); 30],
                    vec![CardId(8001); 3],
                )
            })
            .spawn();
        system
            .entity()
            .component(PlayerId::Player1)
            .component_with(|system| {
                PlayerState::new(
                    system,
                    PlayerId::Player1,
                    vec![CardId(7002); 30],
                    vec![CardId(8001); 3],
                )
            })
            .spawn();

        system.resource_or_default::<DebugLog>().push("游戏开始。");
    }
}

/// 回合开始
pub struct TurnStart {
    pub player: PlayerId,
}

impl Action for TurnStart {
    type Output = ();

    fn perform(&self, system: &mut System) {
        let (player, _) = system.query(exact(self.player)).next().unwrap();
        player.add(system, CurrentTurn);

        system.add_resource(TurnTimer(Timer::new(Duration::from_secs(30))));

        system.resource_or_default::<DebugLog>().push(format!(
            "回合开始，当前为玩家 {} 的回合。",
            self.player as u8,
        ));
    }
}

/// 玩家抽牌
pub struct DrawCards {
    pub player: PlayerId,
    pub count: usize,
}

impl Action for DrawCards {
    type Output = Vec<Entity>;

    fn perform(&self, system: &mut System) -> Self::Output {
        let mut drawn_cards = Vec::new();
        for _ in 0..self.count {
            let (player, _) = system.query(exact(self.player)).next().unwrap();
            let player_state = player.get_mut::<PlayerState>(system).unwrap();
            if let Some(card) = player_state.deck.pop() {
                card.remove::<InDeck>(system);
                let _ = card.add(system, InHand(self.player));
                drawn_cards.push(card);
            }
        }
        system.resource_or_default::<DebugLog>().push(format!(
            "玩家 {} 抽了 {} 张牌。",
            self.player as u8, self.count
        ));
        drawn_cards
    }
}

/// 玩家出牌（开始）
pub struct PlayCard {
    pub player: PlayerId,
    pub card: Entity,
}

impl Action for PlayCard {
    type Output = ();

    fn perform(&self, system: &mut System) -> Self::Output {
        self.card.remove::<InHand>(system);

        system.resource_or_default::<DebugLog>().push(format!(
            "玩家 {} 使用了手牌 {}。",
            self.player as u8,
            self.card.id()
        ));
    }
}

/// 执行卡牌效果
pub struct ExecuteCard {
    pub player: PlayerId,
    pub card_id: CardId,
}

impl Action for ExecuteCard {
    type Output = ();

    fn perform(&self, system: &mut System) {
        let Some(card) = REGISTRY.cards.get(&self.card_id) else {
            return; // 卡牌不存在
        };
        match card {
            CardDef::Order(order_card) => {
                for skill in &order_card.skills {
                    skill(system, self.player);
                }
                system.resource_or_default::<DebugLog>().push(format!(
                    "玩家 {} 执行了卡牌编号 {} 的效果。",
                    self.player as u8, self.card_id.0
                ));
            }
            CardDef::Faith(_) => {}
        }
    }
}

pub struct EndTurn {
    pub player: PlayerId,
}

impl Action for EndTurn {
    type Output = ();

    fn perform(&self, system: &mut System) {
        let (player, _) = system.query(exact(self.player)).next().unwrap();
        player.remove::<CurrentTurn>(system);

        system.remove_resource::<TurnTimer>();

        system
            .resource_or_default::<DebugLog>()
            .push(format!("玩家 {} 结束了回合。", self.player as u8));
    }
}

/// 回合结束，增加回合数
pub struct BumpRound;

impl Action for BumpRound {
    type Output = ();

    fn perform(&self, system: &mut System) {
        let gs = system.resource_mut::<GameState>().unwrap();
        gs.round += 1;

        system
            .resource_or_default::<DebugLog>()
            .push("回合数增加。".to_string());
    }
}

/// 游戏结束
pub struct GameFinished;

impl Action for GameFinished {
    type Output = ();

    fn perform(&self, system: &mut System) {
        let gs = system.resource_mut::<GameState>().unwrap();
        gs.finished = true;

        system
            .resource_or_default::<DebugLog>()
            .push("游戏结束。".to_string());
    }
}
