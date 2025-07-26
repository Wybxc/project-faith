use std::sync::Arc;

use anyhow::Result;
use tokio::task::JoinHandle;

use crate::{
    game::{
        room::Room,
        state::{Action, PlayerId},
    },
    grpc::*,
};

impl Room {
    pub fn main_loop(self: Arc<Self>) -> JoinHandle<Result<()>> {
        tokio::spawn(async move {
            self.perform(Action::Initalize)?;
            loop {
                self.turn(PlayerId::Player0).await?;
                self.turn(PlayerId::Player1).await?;
                self.perform(Action::BumpRound)?;
            }
        })
    }

    async fn turn(self: &Arc<Self>, player: PlayerId) -> Result<()> {
        self.perform(Action::DrawCard(player, 1))?;
        let card_idx = self
            .request_user_event(player, RequestPlayCard {})
            .await?
            .map(|ev| ev.card_idx as usize)
            .unwrap_or(0);
        self.perform(Action::PlayCard(player, card_idx))?;
        Ok(())
    }
}
