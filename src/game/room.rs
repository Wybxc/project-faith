use parking_lot::Mutex;
use tokio::sync::broadcast;
use tonic::Status;

use crate::{
    game::running::RunningGame,
    grpc::{GameEvent, game_event::EventType},
};

pub struct Room {
    p1_sender: broadcast::Sender<GameEvent>,
    p2_sender: broadcast::Sender<GameEvent>,

    pub state: Mutex<RoomState>,
}

impl Room {
    pub fn new(p1_username: String) -> Self {
        let (p1_sender, _) = broadcast::channel(128);
        let (p2_sender, _) = broadcast::channel(128);
        Self {
            p1_sender,
            p2_sender,
            state: Mutex::new(RoomState::Waiting { p1_username }),
        }
    }

    pub fn check_in_room(&self, username: &str) -> bool {
        match &*self.state.lock() {
            RoomState::Waiting { p1_username } if p1_username == username => true,
            RoomState::Playing(running_game) if running_game.is_player(username) => true,
            _ => false,
        }
    }

    pub fn get_sender(&self, username: &str) -> Result<&broadcast::Sender<GameEvent>, Status> {
        match &*self.state.lock() {
            RoomState::Waiting { p1_username } if p1_username == username => Ok(&self.p1_sender),
            RoomState::Playing(rg) if rg.is_player_one(username) => Ok(&self.p1_sender),
            RoomState::Playing(rg) if rg.is_player_two(username) => Ok(&self.p2_sender),
            RoomState::Finished => Err(Status::failed_precondition("Game finished")),
            _ => Err(Status::failed_precondition("Not a player")),
        }
    }

    pub fn sync_game_state(&self) -> Result<(), Status> {
        // Send the current game state to the player
        let state = self.state.lock();
        let RoomState::Playing(game) = &*state else {
            return Ok(()); // No game to sync
        };
        let p1_game_state = game.to_client(true);
        let p2_game_state = game.to_client(false);
        self.p1_sender
            .send(GameEvent {
                event_type: Some(EventType::StateUpdate(p1_game_state)),
            })
            .map_err(|_| Status::internal("Failed to send initial game state"))?;
        self.p2_sender
            .send(GameEvent {
                event_type: Some(EventType::StateUpdate(p2_game_state)),
            })
            .map_err(|_| Status::internal("Failed to send initial game state"))?;
        Ok(())
    }
}

pub enum RoomState {
    Waiting { p1_username: String },
    Playing(RunningGame),
    Finished,
}
