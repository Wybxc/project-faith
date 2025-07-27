use std::{collections::HashMap, sync::LazyLock};

use crate::game::state::{DrawCards, GameState, PlayerId};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct CardId(pub u32);

pub type Skill = Box<dyn Fn(&mut GameState, PlayerId) + Send + Sync>;

pub enum Card {
    Order(OrderCard),
    Faith(FaithCard),
}

/// 指令卡牌
pub struct OrderCard {
    pub card_id: CardId,
    pub name: String,
    pub description: String,
    pub skills: Vec<Skill>,
}

/// 信念卡牌
pub struct FaithCard {
    pub card_id: CardId,
    pub name: String,
    pub description: String,
}

pub struct Registry {
    pub cards: HashMap<CardId, Card>,
}

impl Registry {
    pub fn new() -> Self {
        Self {
            cards: HashMap::new(),
        }
    }

    pub fn order(&mut self, card_id: CardId) -> OrderBuilder {
        OrderBuilder::new(self, card_id)
    }

    pub fn faith(&mut self, card_id: CardId) -> FaithBuilder {
        FaithBuilder::new(self, card_id)
    }
}

pub struct OrderBuilder<'a> {
    registry: &'a mut Registry,
    card_id: CardId,
    name: Option<String>,
    description: Option<String>,
    skills: Vec<Skill>,
}

impl<'a> OrderBuilder<'a> {
    pub fn new(registry: &'a mut Registry, card_id: CardId) -> Self {
        Self {
            registry,
            card_id,
            name: None,
            description: None,
            skills: Vec::new(),
        }
    }

    pub fn skill(mut self, skill: Skill) -> Self {
        self.skills.push(skill);
        self
    }

    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    pub fn description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    pub fn done(self) {
        let order_card = OrderCard {
            card_id: self.card_id,
            name: self.name.expect("Name must be set"),
            description: self.description.expect("Description must be set"),
            skills: self.skills,
        };
        self.registry
            .cards
            .insert(self.card_id, Card::Order(order_card));
    }
}

pub struct FaithBuilder<'a> {
    registry: &'a mut Registry,
    card_id: CardId,
    name: Option<String>,
    description: Option<String>,
}

impl<'a> FaithBuilder<'a> {
    pub fn new(registry: &'a mut Registry, card_id: CardId) -> Self {
        Self {
            registry,
            card_id,
            name: None,
            description: None,
        }
    }

    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    pub fn description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    pub fn done(self) {
        let faith_card = FaithCard {
            card_id: self.card_id,
            name: self.name.expect("Name must be set"),
            description: self.description.expect("Description must be set"),
        };
        self.registry
            .cards
            .insert(self.card_id, Card::Faith(faith_card));
    }
}

fn draw_cards(count: usize) -> Skill {
    Box::new(move |game_state: &mut GameState, player: PlayerId| {
        game_state.perform(DrawCards { player, count });
    })
}

pub static REGISTRY: LazyLock<Registry> = LazyLock::new(|| {
    let mut registry = Registry::new();

    registry
        .order(CardId(7001))
        .name("测试卡牌7001")
        .description("抽一张牌。")
        .skill(draw_cards(1))
        .done();

    registry
        .order(CardId(7002))
        .name("测试卡牌7002")
        .description("抽两张牌。")
        .skill(draw_cards(2))
        .done();

    registry
        .faith(CardId(8001))
        .name("测试信念8001")
        .description("信念卡牌描述。")
        .done();

    registry
});
