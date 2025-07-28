use crate::{game::player::PlayerId, impl_component};

pub mod prototype;

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
