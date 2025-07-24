use std::{collections::HashMap, pin::Pin, sync::Arc};

use base64::prelude::*;
use futures::{Stream, StreamExt};
use parking_lot::Mutex;
use sharded_slab::{Entry, Slab};
use tokio::sync::broadcast;
use tokio_stream::wrappers::BroadcastStream;
use tonic::{Request, Response, Status, async_trait, metadata::MetadataMap};

use crate::{game::running::RunningGame, grpc::*};

mod card;
mod running;

#[derive(Default)]
pub struct Game {
    rooms: Slab<Room>,
    room_map: Arc<Mutex<HashMap<String, usize>>>,
}

impl Game {
    fn auth(&self, metadata: &MetadataMap) -> Result<String, Status> {
        let authentication = metadata
            .get("authentication")
            .ok_or(Status::unauthenticated("Missing authentication token"))?
            .as_bytes();
        let Some(token) = authentication.strip_prefix(b"Bearer ") else {
            return Err(Status::unauthenticated("Invalid authentication token"));
        };
        let token = BASE64_STANDARD
            .decode(token)
            .map_err(|_| Status::unauthenticated("Invalid token format"))?;
        let username = String::from_utf8(token)
            .map_err(|_| Status::unauthenticated("Invalid token encoding"))?;
        Ok(username)
    }

    fn room(&self, room_id: usize) -> Result<Entry<Room>, Status> {
        self.rooms
            .get(room_id)
            .ok_or(Status::internal("Room not found"))
    }
}

#[async_trait]
impl game_service_server::GameService for Game {
    async fn join_room(
        &self,
        request: Request<JoinRoomRequest>,
    ) -> Result<Response<crate::grpc::JoinRoomResponse>, Status> {
        let username = self.auth(request.metadata())?;

        let room_name = request.into_inner().room_name;
        let mut room_map = self.room_map.lock();

        // New room creation
        if !room_map.contains_key(&room_name) {
            let room = Room::new(username);
            let room_id = self.rooms.insert(room).expect("Failed to insert room");
            room_map.insert(room_name.clone(), room_id);
            return Ok(Response::new(JoinRoomResponse {
                message: format!("Created room: {room_name}"),
                room_id: room_id.to_string(),
                success: true,
            }));
        }

        // Joining existing room
        let room_id = *room_map
            .get(&room_name)
            .ok_or_else(|| Status::internal("Room not found in room map"))?;
        let room = self.room(room_id)?;

        let mut state = room.state.lock();
        let p1_username = match &*state {
            RoomState::Waiting { p1_username } if p1_username == &username => {
                return Err(Status::failed_precondition("Already in the room"));
            }
            RoomState::Waiting { p1_username } => p1_username.clone(),
            RoomState::Playing(..) => return Err(Status::failed_precondition("Room is full")),
            RoomState::Finished => return Err(Status::failed_precondition("Room has finished")),
        };
        *state = RoomState::Playing(RunningGame::new(p1_username, username));

        Ok(Response::new(JoinRoomResponse {
            message: format!("Joined room: {room_id}"),
            room_id: room_id.to_string(),
            success: true,
        }))
    }

    type EnterGameStream = Pin<Box<dyn Stream<Item = Result<GameEvent, Status>> + Send>>;

    async fn enter_game(
        &self,
        request: Request<EnterGameRequest>,
    ) -> Result<Response<Self::EnterGameStream>, Status> {
        let _username = self.auth(request.metadata())?;

        let room_id = request.into_inner().room_id;
        let room_id = room_id
            .parse()
            .map_err(|_| Status::invalid_argument("Invalid room ID"))?;
        let room = self.room(room_id)?;

        let events = room.sender.subscribe();
        let events = BroadcastStream::new(events)
            .map(|result| result.map_err(|e| Status::internal(format!("Broadcast error: {e}"))));
        let events = Box::pin(events);
        Ok(Response::new(events))
    }

    async fn ping(&self, request: Request<PingRequest>) -> Result<Response<PingResponse>, Status> {
        let username = self.auth(request.metadata())?;

        let room_id = request.into_inner().room_id;
        tracing::debug!("Received ping for room ID: {}", room_id);
        let room_id = room_id
            .parse()
            .map_err(|_| Status::invalid_argument("Invalid room ID"))?;
        let room = self.room(room_id)?;
        let state = room.state.lock();

        let RoomState::Playing(game) = &*state else {
            return Err(Status::failed_precondition("Room is not in play"));
        };
        let game_state = game.to_client(game.is_player_one(&username));

        room.sender
            .send(GameEvent {
                event_type: Some(game_event::EventType::StateUpdate(game_state)),
            })
            .map_err(|_| Status::internal("Failed to send ping event"))?;

        Ok(Response::new(PingResponse {}))
    }
}

struct Room {
    sender: broadcast::Sender<GameEvent>,
    state: Mutex<RoomState>,
}

impl Room {
    fn new(p1_username: String) -> Self {
        let (sender, _) = broadcast::channel(128);
        Self {
            sender,
            state: Mutex::new(RoomState::Waiting { p1_username }),
        }
    }
}

enum RoomState {
    Waiting { p1_username: String },
    Playing(running::RunningGame),
    Finished,
}
