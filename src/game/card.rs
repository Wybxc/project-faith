use bon::Builder;

use crate::{
    game::{
        action::{DrawCards, Handle},
        player::PlayerId,
    },
    impl_component,
    utils::Map,
};

/// 卡牌 ID
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct CardId(pub u32);
impl_component!(CardId);

/// 卡牌位于玩家手牌中
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct InHand(pub PlayerId);
impl_component!(InHand);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InDeck(pub PlayerId);
impl_component!(InDeck);

/// 卡牌位于玩家信念区
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Faith(pub PlayerId);
impl_component!(Faith);

pub type Skill = Box<dyn Fn(&mut Handle, PlayerId) + Send + Sync>;

pub enum Prototype {
    Order(OrderPrototype),
    Faith(FaithPrototype),
}

/// 指令卡牌
#[derive(Builder)]
#[builder(on(String, into))]
pub struct OrderPrototype {
    #[builder(field)]
    pub skills: Vec<Skill>,
    pub card_id: CardId,
    pub name: String,
    pub description: String,

}

impl<S: order_prototype_builder::State> OrderPrototypeBuilder<S> {
    pub fn skill(mut self, skill: Skill) -> Self {
        self.skills.push(skill);
        self
    }
}

/// 信念卡牌
#[derive(Builder)]
#[builder(on(String, into))]
pub struct FaithPrototype {
    pub card_id: CardId,
    pub name: String,
    pub description: String,
}

pub struct Registry {
    pub cards: Map<CardId, Prototype>,
}

impl Registry {
    pub fn new() -> Self {
        Self { cards: Map::new() }
    }

    pub fn order(&mut self, build: impl FnOnce(OrderPrototypeBuilder) -> OrderPrototype) {
        let card = build(OrderPrototype::builder());
        self.cards.insert(card.card_id, Prototype::Order(card));
    }

    pub fn faith(&mut self, build: impl FnOnce(FaithPrototypeBuilder) -> FaithPrototype) {
        let card = build(FaithPrototype::builder());
        self.cards.insert(card.card_id, Prototype::Faith(card));
    }
}

pub fn draw_cards(count: usize) -> Skill {
    Box::new(move |world, player| {
        world.perform(DrawCards { player, count });
    })
}
