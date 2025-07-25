use std::sync::Arc;

use parking_lot::Mutex;
use tokio::{sync::broadcast, task::JoinHandle};
use tonic::Status;

use crate::{
    game::state::{Action, GameState, PlayerId},
    grpc::{GameEvent, game_event::EventType},
};

pub struct Room {
    p0_sender: broadcast::Sender<GameEvent>,
    p1_sender: broadcast::Sender<GameEvent>,

    pub state: Mutex<RoomState>,
}

impl Room {
    pub fn new(p0_username: String) -> Self {
        let (p0_sender, _) = broadcast::channel(128);
        let (p1_sender, _) = broadcast::channel(128);
        Self {
            p0_sender,
            p1_sender,
            state: Mutex::new(RoomState::Waiting { p0_username }),
        }
    }

    pub fn check_in_room(&self, username: &str) -> bool {
        match &*self.state.lock() {
            RoomState::Waiting { p0_username } if p0_username == username => true,
            RoomState::Playing(running_game) if running_game.is_player(username) => true,
            _ => false,
        }
    }

    pub fn get_sender(&self, username: &str) -> Result<&broadcast::Sender<GameEvent>, Status> {
        match &*self.state.lock() {
            RoomState::Waiting { p0_username } if p0_username == username => Ok(&self.p0_sender),
            RoomState::Playing(rg) if rg.is_player0(username) => Ok(&self.p0_sender),
            RoomState::Playing(rg) if rg.is_player1(username) => Ok(&self.p1_sender),
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
        let p0_game_state = game.to_client(PlayerId::Player0);
        let p1_game_state = game.to_client(PlayerId::Player1);
        let _ = self.p0_sender.send(GameEvent {
            event_type: Some(EventType::StateUpdate(p0_game_state)),
        });
        let _ = self.p1_sender.send(GameEvent {
            event_type: Some(EventType::StateUpdate(p1_game_state)),
        });
        Ok(())
    }

    pub fn perform(&self, action: Action) -> Result<(), Status> {
        let mut state = self.state.lock();
        let RoomState::Playing(game) = &mut *state else {
            return Err(Status::failed_precondition("Game not in progress"));
        };
        game.perform(action);
        drop(state); // Release the lock before sending

        self.sync_game_state()?;
        Ok(())
    }

    pub fn game_start(self: Arc<Self>) -> JoinHandle<Result<(), Status>> {
        tokio::spawn(async move {
            self.perform(Action::Initalize)?;
            loop {
                self.perform(Action::DrawCard(PlayerId::Player0, 1))?;
                self.perform(Action::DrawCard(PlayerId::Player1, 1))?;
                tokio::time::sleep(std::time::Duration::from_secs(1)).await;
                self.perform(Action::BumpRound)?;
            }
        })
    }
}

pub enum RoomState {
    Waiting { p0_username: String },
    Playing(GameState),
    Finished,
}
