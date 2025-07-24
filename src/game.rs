use std::pin::Pin;

use futures::Stream;
use tonic::{Request, Response, Status, async_trait};

use crate::grpc::{
    EnterGameRequest, GameEvent, JoinRoomRequest, JoinRoomResponse,
    game_service_server::GameService,
};

pub struct Game;

#[async_trait]
impl GameService for Game {
    async fn join_room(
        &self,
        request: Request<JoinRoomRequest>,
    ) -> Result<Response<crate::grpc::JoinRoomResponse>, Status> {
        request
            .metadata()
            .get("authorization")
            .ok_or(Status::unauthenticated("Missing token"))?;
        let room_id = request.into_inner().room_id;
        Ok(Response::new(JoinRoomResponse {
            message: format!("Joined room: {room_id}"),
            success: true,
        }))
    }

    type EnterGameStream = Pin<Box<dyn Stream<Item = Result<GameEvent, Status>> + Send>>;

    async fn enter_game(
        &self,
        request: Request<EnterGameRequest>,
    ) -> Result<Response<Self::EnterGameStream>, Status> {
        request
            .metadata()
            .get("authorization")
            .ok_or(Status::unauthenticated("Missing token"))?;
        let _room_id = request.into_inner().room_id;
        let events = futures::stream::iter(vec![Ok(GameEvent {
            event_type: "game_started".to_string(),
            data: "Welcome to the game!".to_string(),
        })]);
        let events = Box::pin(events);
        Ok(Response::new(events))
    }
}
