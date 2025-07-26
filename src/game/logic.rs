use std::sync::Arc;

use tokio::task::JoinHandle;
use tonic::Status;

use crate::{
    game::{
        room::Room,
        state::{Action, PlayerId},
    },
    grpc::*,
};

impl Room {
    pub fn main_loop(self: Arc<Self>) -> JoinHandle<Result<(), Status>> {
        tokio::spawn(async move {
            self.perform(Action::Initalize)?;
            loop {
                self.perform(Action::DrawCard(PlayerId::Player0, 1))?;
                self.request_user_event(PlayerId::Player0, RequestPlayCard {}).await?;

                self.perform(Action::DrawCard(PlayerId::Player1, 1))?;
                self.request_user_event(PlayerId::Player1, RequestPlayCard {}).await?;

                self.perform(Action::BumpRound)?;
            }
        })
    }
}
