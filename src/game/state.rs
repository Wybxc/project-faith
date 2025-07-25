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

    fn me(&self, player: PlayerId) -> &PlayerState {
        match player {
            PlayerId::Player0 => &self.players.0,
            PlayerId::Player1 => &self.players.1,
        }
    }

    fn me_mut(&mut self, player: PlayerId) -> &mut PlayerState {
        match player {
            PlayerId::Player0 => &mut self.players.0,
            PlayerId::Player1 => &mut self.players.1,
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

    pub fn to_client(&self, player: PlayerId) -> grpc::GameState {
        let self_hand = self.me(player).hand.iter().map(|id| id.0).collect();
        let other_hand_count = self.me(player.opp()).hand.len() as u32;
        let self_deck_count = self.me(player).deck.len() as u32;
        let other_deck_count = self.me(player.opp()).deck.len() as u32;

        grpc::GameState {
            self_hand,
            other_hand_count,
            self_deck_count,
            other_deck_count,
        }
    }

    /// Applies an action to the game state.
    pub fn reduce(&mut self, action: Action) {
        match action {
            Action::DrawCard(player) => {
                let player_state = self.me_mut(player);
                if let Some(card) = player_state.deck.pop() {
                    player_state.hand.push(card);
                }
            }
        }
    }
}

pub enum Action {
    /// Draw a card from the deck
    DrawCard(PlayerId),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlayerId {
    Player0 = 0,
    Player1 = 1,
}

impl PlayerId {
    /// Returns the opposite player ID.
    pub fn opp(self) -> PlayerId {
        match self {
            PlayerId::Player0 => PlayerId::Player1,
            PlayerId::Player1 => PlayerId::Player0,
        }
    }
}
