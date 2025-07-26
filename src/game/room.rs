use std::sync::Arc;

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

pub enum RoomState {
    Waiting { p0_username: String },
    Playing(GameState),
    Finished,
}

pub struct Room {
    p0_sender: broadcast::Sender<GameEvent>,
    p1_sender: broadcast::Sender<GameEvent>,

    /// Waiting user events
    ///
    /// Actually no more than 2 user events are expected at the same time
    /// (one for each player). Maybe we can use a more efficient data structure?
    user_events: Slab<oneshot::Sender<user_event::EventType>>,

    /// Pending events will be re-sent to the player if they re-join the room
    p0_pending_event: Mutex<Option<RequestUserEvent>>,
    /// Pending events will be re-sent to the player if they re-join the room
    p1_pending_event: Mutex<Option<RequestUserEvent>>,

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
            p0_pending_event: Mutex::new(None),
            p1_pending_event: Mutex::new(None),
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
}

#[allow(clippy::result_large_err)]
impl Room {
    pub fn get_player(&self, username: &str) -> Result<PlayerId, Status> {
        match &*self.state.lock() {
            RoomState::Waiting { p0_username } if p0_username == username => Ok(PlayerId::Player0),
            RoomState::Playing(rg) if rg.is_player0(username) => Ok(PlayerId::Player0),
            RoomState::Playing(rg) if rg.is_player1(username) => Ok(PlayerId::Player1),
            _ => Err(Status::failed_precondition("Not a player in this room")),
        }
    }

    pub fn get_sender(&self, username: &str) -> Result<&broadcast::Sender<GameEvent>, Status> {
        match self.get_player(username)? {
            PlayerId::Player0 => Ok(&self.p0_sender),
            PlayerId::Player1 => Ok(&self.p1_sender),
        }
    }

    pub fn send_pending_event(&self, username: &str) -> Result<(), Status> {
        let player = self.get_player(username)?;
        let pending_event = match player {
            PlayerId::Player0 => self.p0_pending_event.lock(),
            PlayerId::Player1 => self.p1_pending_event.lock(),
        };
        let Some(request) = pending_event.as_ref() else {
            return Ok(()); // No pending event to send
        };

        let event_sender = match player {
            PlayerId::Player0 => &self.p0_sender,
            PlayerId::Player1 => &self.p1_sender,
        };
        let _ = event_sender.send(GameEvent {
            event_type: Some(game_event::EventType::RequestUserEvent(*request)),
        });

        Ok(())
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

impl Room {
    pub fn read_state<T>(&self, reader: impl FnOnce(&GameState) -> T) -> anyhow::Result<T> {
        let state = self.state.lock();
        match &*state {
            RoomState::Playing(game) => Ok(reader(game)),
            _ => Err(anyhow::anyhow!("Game not in progress")),
        }
    }

    pub fn sync_game_state(&self) {
        // Send the current game state to the player
        let state = self.state.lock();
        let RoomState::Playing(game) = &*state else {
            return; // No game to sync
        };
        let p0_game_state = game.to_client(PlayerId::Player0);
        let p1_game_state = game.to_client(PlayerId::Player1);
        let _ = self.p0_sender.send(GameEvent {
            event_type: Some(game_event::EventType::StateUpdate(p0_game_state)),
        });
        let _ = self.p1_sender.send(GameEvent {
            event_type: Some(game_event::EventType::StateUpdate(p1_game_state)),
        });
    }

    pub fn perform<A: Action>(&self, action: A) -> anyhow::Result<A::Output> {
        let output = {
            let mut state = self.state.lock();
            let RoomState::Playing(game) = &mut *state else {
                return Err(anyhow::anyhow!("Game not in progress"));
            };
            game.perform(action)
        };

        self.sync_game_state();
        Ok(output)
    }

    pub async fn request_user_event<E: UserEvent>(
        self: &Arc<Self>,
        player: PlayerId,
        request: E,
    ) -> anyhow::Result<Option<E::Response>> {
        let event_sender = match player {
            PlayerId::Player0 => &self.p0_sender,
            PlayerId::Player1 => &self.p1_sender,
        };

        let (sender, receiver) = oneshot::channel();
        let seqnum = self
            .user_events
            .insert(sender)
            .expect("Failed to insert user event sender");

        let timeout = 20;

        let request = RequestUserEvent {
            seqnum: seqnum as u64,
            timeout,
            event_type: Some(request.into_rpc()),
        };
        let _ = event_sender.send(GameEvent {
            event_type: Some(game_event::EventType::RequestUserEvent(request)),
        });

        {
            let mut pending_event = match player {
                PlayerId::Player0 => self.p0_pending_event.lock(),
                PlayerId::Player1 => self.p1_pending_event.lock(),
            };
            *pending_event = Some(request);
        }

        let this = Arc::clone(self);
        let countdown = tokio::spawn(async move {
            loop {
                tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
                let mut pending_event = match player {
                    PlayerId::Player0 => this.p0_pending_event.lock(),
                    PlayerId::Player1 => this.p1_pending_event.lock(),
                };
                if let Some(req) = pending_event.as_mut() {
                    req.timeout -= 1;
                    // Client timeout is 1 second shorter than server timeout
                    if req.timeout <= -1 {
                        break; // Timeout reached, exit countdown
                    }
                } else {
                    break; // No pending event, exit countdown
                }
            }
        });
        let response = tokio::select! {
            response = receiver => {
                if let Ok(event_type) = response {
                    Ok(Some(E::from_rpc(event_type)?))
                } else {
                    Ok(None)
                }
            }
            _ = countdown => {
                Ok(None) // Timeout reached, return None
            }
        };

        {
            let mut pending_event = match player {
                PlayerId::Player0 => self.p0_pending_event.lock(),
                PlayerId::Player1 => self.p1_pending_event.lock(),
            };
            *pending_event = None; // Clear the pending event after response or timeout
        }

        response
    }
}
