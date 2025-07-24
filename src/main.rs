use tonic::transport::Server;
use tonic_web::GrpcWebLayer;

use crate::grpc::auth_service_server::AuthServiceServer;

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
        .layer(GrpcWebLayer::new())
        .add_service(AuthServiceServer::new(auth::Auth))
        .serve("[::1]:8617".parse().unwrap());
    grpc_server.await.unwrap();
}
