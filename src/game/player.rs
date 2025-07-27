use crate::game::card::CardId;

#[derive(Default)]
pub struct PlayerState {
    /// The player's hand of cards, from left to right.
    pub hand: Vec<CardId>,
    /// The player's deck of cards, from bottom to top.
    pub deck: Vec<CardId>,
}

impl PlayerState {
    pub fn initialize(&mut self, deck: Vec<CardId>) {
        self.hand.clear();
        self.deck = deck;
    }
}
