use crate::game::card::CardId;

pub struct PlayerState {
    pub username: String,
    pub hand: Vec<CardId>,
    pub deck: Vec<CardId>,
}

impl PlayerState {
    pub fn new(username: String) -> Self {
        Self {
            username,
            hand: Vec::new(),
            deck: vec![CardId(7001); 30], // Example deck with 30 cards
        }
    }
}
