use crate::{game::card::CardId, grpc::GameState};

pub struct RunningGame {
    /// Player 1's username
    p1_username: String,
    /// Player 2's username
    p2_username: String,

    /// Player 1's hand of cards
    p1_hand: Vec<CardId>,
    /// Player 2's hand of cards
    p2_hand: Vec<CardId>,
}

impl RunningGame {
    pub fn new(p1_username: String, p2_username: String) -> Self {
        Self {
            p1_username,
            p2_username,
            p1_hand: Vec::new(),
            p2_hand: Vec::new(),
        }
    }

    fn self_hand(&self, is_player_one: bool) -> &[CardId] {
        if is_player_one {
            &self.p1_hand
        } else {
            &self.p2_hand
        }
    }

    fn other_hand(&self, is_player_one: bool) -> &[CardId] {
        if is_player_one {
            &self.p2_hand
        } else {
            &self.p1_hand
        }
    }

    pub fn is_player_one(&self, username: &str) -> bool {
        self.p1_username == username
    }

    pub fn to_client(&self, is_player_one: bool) -> GameState {
        let self_hand = self
            .self_hand(is_player_one)
            .iter()
            .map(|id| id.0)
            .collect();
        let other_hand_count = self.other_hand(is_player_one).len() as u32;

        GameState {
            self_hand,
            other_hand_count,
        }
    }
}
