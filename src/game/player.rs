use crate::game::card::CardId;

pub struct PlayerState {
    pub username: String,
    /// The player's hand of cards, from left to right.
    pub hand: Vec<CardId>,
    /// The player's deck of cards, from bottom to top.
    pub deck: Vec<CardId>,
}

impl PlayerState {
    pub fn new(username: String) -> Self {
        Self {
            username,
            hand: Vec::new(),
            deck: Vec::new(),
        }
    }

    pub fn initialize(&mut self, deck: Vec<CardId>) {
        self.hand.clear();
        self.deck = deck;
    }
}
