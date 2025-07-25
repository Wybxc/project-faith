use crate::{game::player::PlayerState, grpc};

pub struct GameState {
    players: (PlayerState, PlayerState),
}

impl GameState {
    pub fn new(p0_username: String, p1_username: String) -> Self {
        let p0 = PlayerState::new(p0_username);
        let p1 = PlayerState::new(p1_username);
        Self { players: (p0, p1) }
    }

    fn me(&self, is_player0: bool) -> &PlayerState {
        if is_player0 {
            &self.players.0
        } else {
            &self.players.1
        }
    }

    fn other(&self, is_player1: bool) -> &PlayerState {
        if is_player1 {
            &self.players.1
        } else {
            &self.players.0
        }
    }

    pub fn is_player(&self, username: &str) -> bool {
        self.is_player0(username) || self.is_player1(username)
    }

    pub fn is_player0(&self, username: &str) -> bool {
        self.players.0.username == username
    }

    pub fn is_player1(&self, username: &str) -> bool {
        self.players.1.username == username
    }

    pub fn to_client(&self, is_player0: bool) -> grpc::GameState {
        let self_hand = self.me(is_player0).hand.iter().map(|id| id.0).collect();
        let other_hand_count = self.other(is_player0).hand.len() as u32;
        let self_deck_count = self.me(is_player0).deck.len() as u32;
        let other_deck_count = self.other(is_player0).deck.len() as u32;

        grpc::GameState {
            self_hand,
            other_hand_count,
            self_deck_count,
            other_deck_count,
        }
    }
}
