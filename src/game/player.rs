use crate::{
    game::card::{CardId, Faith, InDeck},
    impl_component,
    system::{Entity, World},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PlayerId {
    Player0 = 0,
    Player1 = 1,
}
impl_component!(PlayerId);

impl PlayerId {
    /// Returns the opposite player ID.
    pub fn opp(self) -> PlayerId {
        match self {
            PlayerId::Player0 => PlayerId::Player1,
            PlayerId::Player1 => PlayerId::Player0,
        }
    }
}

#[derive(Default)]
pub struct PlayerState {
    /// 玩家卡组实体列表
    pub deck: Vec<Entity>,
}
impl_component!(PlayerState);

impl PlayerState {
    pub fn new(world: &mut World, player: PlayerId, deck: Vec<CardId>, faith: Vec<CardId>) -> Self {
        let deck = deck
            .into_iter()
            .map(|card_id| {
                world
                    .entity()
                    .component(card_id)
                    .component(InDeck(player))
                    .spawn()
            })
            .collect();
        for card_id in faith {
            world
                .entity()
                .component(card_id)
                .component(Faith(player))
                .spawn();
        }
        Self { deck }
    }
}

pub struct CurrentTurn;
impl_component!(CurrentTurn);
