use std::{collections::HashMap, pin::Pin, sync::Arc};

use base64::prelude::*;
use futures::{Stream, StreamExt};
use parking_lot::{Mutex, RwLock};
use sharded_slab::{Entry, Slab};
use tokio::sync::broadcast;
use tokio_stream::wrappers::BroadcastStream;
use tonic::{Request, Response, Status, async_trait, metadata::MetadataMap};

use crate::grpc::*;

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
        let _username = self.auth(request.metadata())?;

        let room_name = request.into_inner().room_name;
        let mut room_map = self.room_map.lock();
        let room_id = *room_map.entry(room_name).or_insert_with(|| {
            let room = Room::new();
            self.rooms.insert(room).expect("Failed to insert room")
        });
        let room = self.room(room_id)?;

        let state = room.state.read();
        match &*state {
            RoomState::Playing => return Err(Status::failed_precondition("Room is full")),
            RoomState::Finished => return Err(Status::failed_precondition("Room has finished")),
            _ => {}
        }
        drop(state); // Release the read lock before acquiring a write lock
        *room.state.write() = RoomState::Playing;

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
        let _username = self.auth(request.metadata())?;

        let room_id = request.into_inner().room_id;
        tracing::debug!("Received ping for room ID: {}", room_id);
        let room_id = room_id
            .parse()
            .map_err(|_| Status::invalid_argument("Invalid room ID"))?;
        let room = self.room(room_id)?;
        room.sender
            .send(GameEvent {
                event_type: "ping".to_string(),
                data: "Ping received".to_string(),
            })
            .map_err(|_| Status::internal("Failed to send ping event"))?;

        Ok(Response::new(PingResponse {}))
    }
}

struct Room {
    sender: broadcast::Sender<GameEvent>,
    state: RwLock<RoomState>,
}

impl Room {
    fn new() -> Self {
        let (sender, _) = broadcast::channel(128);
        Self {
            sender,
            state: RwLock::new(RoomState::Waiting),
        }
    }
}

enum RoomState {
    Waiting,
    Playing,
    Finished,
}
