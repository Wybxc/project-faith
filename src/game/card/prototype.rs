use std::sync::LazyLock;

use crate::{
    game::{
        action::{DrawCards, Handle},
        card::CardId,
        player::PlayerId,
    },
    utils::Map,
};

pub type Skill = Box<dyn Fn(&mut Handle, PlayerId) + Send + Sync>;

pub enum Prototype {
    Order(OrderPrototype),
    Faith(FaithPrototype),
}

/// 指令卡牌
pub struct OrderPrototype {
    pub card_id: CardId,
    pub skills: Vec<Skill>,
}

/// 信念卡牌
pub struct FaithPrototype {
    pub card_id: CardId,
}

pub struct Registry {
    pub cards: Map<CardId, Prototype>,
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
        let order_card = OrderPrototype {
            card_id: self.card_id,
            skills: self.skills,
        };
        self.registry
            .cards
            .insert(self.card_id, Prototype::Order(order_card));
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
        let faith_card = FaithPrototype {
            card_id: self.card_id,
        };
        self.registry
            .cards
            .insert(self.card_id, Prototype::Faith(faith_card));
    }
}

fn draw_cards(count: usize) -> Skill {
    Box::new(move |world, player| {
        world.perform(DrawCards { player, count });
    })
}

pub static REGISTRY: LazyLock<Registry> = LazyLock::new(|| {
    let mut registry = Registry::new();

    registry.order(CardId(7001)).skill(draw_cards(1)).done();

    registry.order(CardId(7002)).skill(draw_cards(2)).done();

    registry.faith(CardId(8001)).done();

    registry
});
