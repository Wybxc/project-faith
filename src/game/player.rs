use crate::game::card::CardId;

#[derive(Default)]
pub struct PlayerState {
    /// The player's hand of cards, from left to right.
    pub hand: Vec<CardId>,
    /// The player's deck of cards, from bottom to top.
    pub deck: Vec<CardId>,
    /// Faith cards
    pub faith: Vec<CardId>,
}

impl PlayerState {
    pub fn initialize(&mut self, deck: Vec<CardId>, faith_cards: Vec<CardId>) {
        self.hand.clear();
        self.deck = deck;
        self.faith = faith_cards;
    }
}
