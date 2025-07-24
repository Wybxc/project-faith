#![allow(clippy::result_large_err)]

use http::HeaderName;
use tonic::transport::Server;
use tonic_web::GrpcWebLayer;
use tower_http::cors::{Any, CorsLayer};

use crate::grpc::{auth_service_server::AuthServiceServer, game_service_server::GameServiceServer};

mod auth;
mod game;

mod grpc {
    tonic::include_proto!("auth.v1");
    tonic::include_proto!("game.v1");
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .init();

    let grpc_server = Server::builder()
        .accept_http1(true)
        .layer(
            CorsLayer::new()
                .allow_origin(Any)
                .allow_headers(Any)
                .allow_methods(Any)
                .expose_headers([
                    HeaderName::from_static("grpc-status"),
                    HeaderName::from_static("grpc-message"),
                ]),
        )
        .layer(GrpcWebLayer::new())
        .add_service(AuthServiceServer::new(auth::Auth))
        .add_service(GameServiceServer::new(game::Game::default()))
        .serve("[::1]:8617".parse().unwrap());

    tracing::info!("gRPC server listening on [::1]:8617");
    grpc_server.await.unwrap();
}
