use std::sync::LazyLock;

use crate::{
    game::state::{DrawCards, GameState, PlayerId},
    system::{Component, Entity},
    utils::Map,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct CardId(pub u32);

impl Component for CardId {
    type Storage = Map<Entity, CardId>;
}

pub type Skill = Box<dyn Fn(&mut GameState, PlayerId) + Send + Sync>;

pub enum CardDef {
    Order(OrderCardDef),
    Faith(FaithCardDef),
}

/// 指令卡牌
pub struct OrderCardDef {
    pub card_id: CardId,
    pub skills: Vec<Skill>,
}

/// 信念卡牌
pub struct FaithCardDef {
    pub card_id: CardId,
}

pub struct Registry {
    pub cards: Map<CardId, CardDef>,
}

impl Registry {
    pub fn new() -> Self {
        Self { cards: Map::new() }
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
    skills: Vec<Skill>,
}

impl<'a> OrderBuilder<'a> {
    pub fn new(registry: &'a mut Registry, card_id: CardId) -> Self {
        Self {
            registry,
            card_id,
            skills: Vec::new(),
        }
    }

    pub fn skill(mut self, skill: Skill) -> Self {
        self.skills.push(skill);
        self
    }

    pub fn done(self) {
        let order_card = OrderCardDef {
            card_id: self.card_id,
            skills: self.skills,
        };
        self.registry
            .cards
            .insert(self.card_id, CardDef::Order(order_card));
    }
}

pub struct FaithBuilder<'a> {
    registry: &'a mut Registry,
    card_id: CardId,
}

impl<'a> FaithBuilder<'a> {
    pub fn new(registry: &'a mut Registry, card_id: CardId) -> Self {
        Self { registry, card_id }
    }

    pub fn done(self) {
        let faith_card = FaithCardDef {
            card_id: self.card_id,
        };
        self.registry
            .cards
            .insert(self.card_id, CardDef::Faith(faith_card));
    }
}

fn draw_cards(count: usize) -> Skill {
    Box::new(move |game_state: &mut GameState, player: PlayerId| {
        game_state.perform(DrawCards { player, count });
    })
}

pub static REGISTRY: LazyLock<Registry> = LazyLock::new(|| {
    let mut registry = Registry::new();

    registry.order(CardId(7001)).skill(draw_cards(1)).done();

    registry.order(CardId(7002)).skill(draw_cards(2)).done();

    registry.faith(CardId(8001)).done();

    registry
});
