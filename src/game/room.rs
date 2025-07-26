use parking_lot::Mutex;
use sharded_slab::Slab;
use tokio::sync::{broadcast, oneshot};
use tonic::Status;

use crate::{
    game::{
        state::{Action, GameState, PlayerId},
        user::UserEvent,
    },
    grpc::*,
};

pub struct Room {
    p0_sender: broadcast::Sender<GameEvent>,
    p1_sender: broadcast::Sender<GameEvent>,

    user_events: Slab<oneshot::Sender<user_event::EventType>>,

    pub state: Mutex<RoomState>,
}

impl Room {
    pub fn new(p0_username: String) -> Self {
        let (p0_sender, _) = broadcast::channel(128);
        let (p1_sender, _) = broadcast::channel(128);
        Self {
            p0_sender,
            p1_sender,
            user_events: Slab::new(),
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
            event_type: Some(game_event::EventType::StateUpdate(p0_game_state)),
        });
        let _ = self.p1_sender.send(GameEvent {
            event_type: Some(game_event::EventType::StateUpdate(p1_game_state)),
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

    pub async fn request_user_event<E: UserEvent>(
        &self,
        player: PlayerId,
        request: E,
    ) -> Result<Option<E::Response>, Status> {
        let event_sender = match player {
            PlayerId::Player0 => &self.p0_sender,
            PlayerId::Player1 => &self.p1_sender,
        };

        let (sender, receiver) = oneshot::channel();
        let seqnum = self
            .user_events
            .insert(sender)
            .expect("Failed to insert user event sender");

        let _ = event_sender.send(GameEvent {
            event_type: Some(game_event::EventType::RequestUserEvent(RequestUserEvent {
                seqnum: seqnum as u64,
                event_type: Some(request.into_rpc()),
            })),
        });

        tokio::select! {
            response = receiver => {
                if let Ok(event_type) = response {
                    Ok(Some(E::from_rpc(event_type)?))
                } else {
                    Ok(None)
                }
            }
            _ = tokio::time::sleep(std::time::Duration::from_secs(21)) => {
                Ok(None) // Timeout
            }
        }
    }

    pub fn submit_user_event(
        &self,
        seqnum: usize,
        event_type: user_event::EventType,
    ) -> Result<(), Status> {
        let Some(ch) = self.user_events.take(seqnum) else {
            return Err(Status::not_found("User event not found"));
        };
        let _ = ch.send(event_type);
        Ok(())
    }
}

pub enum RoomState {
    Waiting { p0_username: String },
    Playing(GameState),
    Finished,
}
