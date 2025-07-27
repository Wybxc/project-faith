#![allow(dead_code)]

use http::HeaderName;
use tonic::transport::Server;
use tonic_web::GrpcWebLayer;
use tower_http::cors::{Any, CorsLayer};

use crate::grpc::*;

mod auth;
mod system;
mod game;
mod utils;

mod grpc {
    tonic::include_proto!("auth.v1");
    tonic::include_proto!("game.v1");
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .init();

    std::thread::spawn(move || {
        loop {
            std::thread::sleep(std::time::Duration::from_secs(10));
            let deadlocks = parking_lot::deadlock::check_deadlock();
            if deadlocks.is_empty() {
                continue;
            }

            println!("{} deadlocks detected", deadlocks.len());
            for (i, threads) in deadlocks.iter().enumerate() {
                println!("Deadlock #{i}");
                for t in threads {
                    println!("Thread Id {:#?}", t.thread_id());
                    println!("{:#?}", t.backtrace());
                }
            }
        }
    });

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
        .add_service(auth_service_server::AuthServiceServer::new(auth::Auth))
        .add_service(game_service_server::GameServiceServer::new(
            game::Game::default(),
        ))
        .serve("[::1]:8617".parse().unwrap());

    tracing::info!("gRPC server listening on [::1]:8617");
    grpc_server.await.unwrap();
}
