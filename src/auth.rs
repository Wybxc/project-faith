use base64::prelude::*;
use tonic::{Request, Response, Status, async_trait};

use crate::grpc::*;

pub struct Auth;

#[async_trait]
impl auth_service_server::AuthService for Auth {
    async fn login(
        &self,
        request: Request<LoginRequest>,
    ) -> Result<Response<LoginResponse>, Status> {
        let username = request.into_inner().username;
        let token = BASE64_STANDARD.encode(username.as_bytes());
        tracing::info!("User {} logged in with token {}", username, token);
        Ok(Response::new(LoginResponse {
            message: format!("Hello, {username}!"),
            token,
        }))
    }
}
