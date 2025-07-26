use std::{
    collections::HashMap,
    pin::Pin,
    sync::{Arc, atomic::Ordering},
};

use base64::prelude::*;
use futures::{Stream, StreamExt};
use parking_lot::Mutex;
use sharded_slab::{Entry, Slab};
use tokio_stream::wrappers::BroadcastStream;
use tonic::{Request, Response, Status, async_trait, metadata::MetadataMap};

use crate::{
    game::room::{Room, RoomState},
    grpc::*,
};

mod card;
mod logic;
mod player;
mod room;
mod state;
mod user;

#[derive(Default)]
pub struct Game {
    rooms: Slab<Arc<Room>>,
    room_map: Arc<Mutex<HashMap<String, usize>>>,
}

#[allow(clippy::result_large_err)]
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

    fn room(&self, room_id: usize) -> Result<Entry<Arc<Room>>, Status> {
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
            let room = Room::new(username.clone());
            let room_id = self
                .rooms
                .insert(Arc::new(room))
                .expect("Failed to insert room");
            room_map.insert(room_name.clone(), room_id);
            tracing::info!("Created new room: {}, player: {}", room_name, username);
            return Ok(Response::new(JoinRoomResponse {
                message: format!("Created room: {room_name}"),
                room_id: room_id as u64,
                success: true,
            }));
        }

        // Joining existing room
        let room_id = *room_map
            .get(&room_name)
            .ok_or_else(|| Status::internal("Room not found in room map"))?;
        let room = self.room(room_id)?;

        if room.check_in_room(&username) {
            return Ok(Response::new(JoinRoomResponse {
                message: "Already in the room".to_string(),
                room_id: room_id as u64,
                success: true,
            }));
        }

        if let Err(state) = room.room_state.compare_exchange(
            RoomState::Waiting,
            RoomState::Playing,
            Ordering::SeqCst,
            Ordering::SeqCst,
        ) {
            return Err(match state {
                RoomState::Waiting => unreachable!(),
                RoomState::Playing => Status::failed_precondition("Room is full"),
                RoomState::Finished => Status::failed_precondition("Room has finished"),
            });
        };
        room.set_player1(username.clone())?;
        tracing::info!("Player {} joined room: {}", username, room_name);

        let _handle = room.clone().main_loop(); // TODO: avoid memory leak

        Ok(Response::new(JoinRoomResponse {
            message: format!("Joined room: {room_id}"),
            room_id: room_id as u64,
            success: true,
        }))
    }

    type EnterGameStream = Pin<Box<dyn Stream<Item = Result<GameEvent, Status>> + Send>>;

    async fn enter_game(
        &self,
        request: Request<EnterGameRequest>,
    ) -> Result<Response<Self::EnterGameStream>, Status> {
        let username = self.auth(request.metadata())?;

        let room_id = request.into_inner().room_id as usize;
        let room = self.room(room_id)?;

        let events = room.get_sender(&username)?.subscribe();
        let events = BroadcastStream::new(events)
            .map(|result| result.map_err(|e| Status::internal(format!("Broadcast error: {e}"))));
        let events = Box::pin(events);

        room.send_pending_event(&username)?;
        room.sync_game_state();

        Ok(Response::new(events))
    }

    async fn submit_user_event(
        &self,
        request: Request<UserEvent>,
    ) -> Result<Response<UserEventResponse>, Status> {
        let _username = self.auth(request.metadata())?;

        let request = request.into_inner();
        let room_id = request.room_id as usize;
        let room = self.room(room_id)?;

        let Some(event) = request.event_type else {
            return Err(Status::invalid_argument("Event type is required"));
        };
        room.submit_user_event(request.seqnum as usize, event)?;
        Ok(Response::new(UserEventResponse {}))
    }
}
