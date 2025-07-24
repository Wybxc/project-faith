use tonic::{Request, Response, Status, async_trait};

use crate::grpc::{LoginRequest, LoginResponse, auth_service_server::AuthService};

pub struct Auth;

#[async_trait]
impl AuthService for Auth {
    async fn login(
        &self,
        request: Request<LoginRequest>,
    ) -> Result<Response<LoginResponse>, Status> {
        let username = request.into_inner().username;
        Ok(Response::new(LoginResponse {
            message: format!("Hello, {username}!"),
            token: username,
        }))
    }
}
