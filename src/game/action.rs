use std::time::Duration;

use crate::{
    game::{
        card::{
            CardId, InDeck, InHand,
            prototype::{Prototype, REGISTRY},
        },
        player::{CurrentTurn, PlayerId, PlayerState},
        state::{DebugLog, GlobalState, TurnTimer},
    },
    system::*,
    utils::Timer,
};

pub struct Handle<'a>(&'a mut World);

impl<'a> Handle<'a> {
    pub fn perform<A: Action>(&mut self, action: A) -> A::Output {
        action.perform(self.0)
    }
}

pub trait Action {
    type Output;

    fn perform(&self, world: &mut World) -> Self::Output;
}

/// 初始化游戏状态
pub struct Initalize;

impl Action for Initalize {
    type Output = ();

    fn perform(&self, world: &mut World) {
        world.add_resource(GlobalState::new());

        world
            .entity()
            .component(PlayerId::Player0)
            .component_with(|world| {
                PlayerState::new(
                    world,
                    PlayerId::Player0,
                    vec![CardId(7001); 30],
                    vec![CardId(8001); 3],
                )
            })
            .spawn();
        world
            .entity()
            .component(PlayerId::Player1)
            .component_with(|world| {
                PlayerState::new(
                    world,
                    PlayerId::Player1,
                    vec![CardId(7002); 30],
                    vec![CardId(8001); 3],
                )
            })
            .spawn();

        world.resource_or_default::<DebugLog>().push("游戏开始。");
    }
}

/// 回合开始
pub struct StartTurn {
    pub player: PlayerId,
}

impl Action for StartTurn {
    type Output = ();

    fn perform(&self, world: &mut World) {
        let (player, _) = world.query(exact(self.player)).next().unwrap();
        player.add(world, CurrentTurn);

        world.add_resource(TurnTimer(Timer::new(Duration::from_secs(30))));

        world.resource_or_default::<DebugLog>().push(format!(
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

    fn perform(&self, world: &mut World) -> Self::Output {
        let mut drawn_cards = Vec::new();
        for _ in 0..self.count {
            let (player, _) = world.query(exact(self.player)).next().unwrap();
            let player_state = player.get_mut::<PlayerState>(world).unwrap();
            if let Some(card) = player_state.deck.pop() {
                card.remove::<InDeck>(world);
                let _ = card.add(world, InHand(self.player));
                drawn_cards.push(card);
            }
        }
        world.resource_or_default::<DebugLog>().push(format!(
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

    fn perform(&self, world: &mut World) -> Self::Output {
        self.card.remove::<InHand>(world);

        world.resource_or_default::<DebugLog>().push(format!(
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

    fn perform(&self, world: &mut World) {
        let Some(card) = REGISTRY.cards.get(&self.card_id) else {
            return; // 卡牌不存在
        };
        match card {
            Prototype::Order(order_card) => {
                for skill in &order_card.skills {
                    skill(&mut Handle(world), self.player);
                }
                world.resource_or_default::<DebugLog>().push(format!(
                    "玩家 {} 执行了卡牌编号 {} 的效果。",
                    self.player as u8, self.card_id.0
                ));
            }
            Prototype::Faith(_) => {}
        }
    }
}

pub struct EndTurn {
    pub player: PlayerId,
}

impl Action for EndTurn {
    type Output = ();

    fn perform(&self, world: &mut World) {
        let (player, _) = world.query(exact(self.player)).next().unwrap();
        player.remove::<CurrentTurn>(world);

        world.remove_resource::<TurnTimer>();

        world
            .resource_or_default::<DebugLog>()
            .push(format!("玩家 {} 结束了回合。", self.player as u8));
    }
}

/// 回合结束，增加回合数
pub struct BumpRound;

impl Action for BumpRound {
    type Output = ();

    fn perform(&self, world: &mut World) {
        let gs = world.resource_mut::<GlobalState>().unwrap();
        gs.round += 1;

        world
            .resource_or_default::<DebugLog>()
            .push("回合数增加。".to_string());
    }
}

/// 游戏结束
pub struct GameFinished;

impl Action for GameFinished {
    type Output = ();

    fn perform(&self, world: &mut World) {
        let gs = world.resource_mut::<GlobalState>().unwrap();
        gs.finished = true;

        world
            .resource_or_default::<DebugLog>()
            .push("游戏结束。".to_string());
    }
}
