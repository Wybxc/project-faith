use std::sync::Arc;

use anyhow::Result;
use tokio::task::JoinHandle;

use crate::{
    game::{room::Room, state::*},
    grpc::RequestPlayCard,
};

impl Room {
    pub fn main_loop(self: Arc<Self>) -> JoinHandle<Result<()>> {
        tokio::spawn(async move {
            self.perform(Initalize)?;
            loop {
                self.turn(PlayerId::Player0).await?;
                self.turn(PlayerId::Player1).await?;
                self.perform(BumpRound)?;
            }
        })
    }

    async fn turn(self: &Arc<Self>, player: PlayerId) -> Result<()> {
        self.perform(DrawCards { player, count: 1 })?;

        if self.read_state(|gs| gs.me(player).hand.len())? > 0 {
            // 如果玩家有手牌，则请求出牌
            let card_index = self
                .request_user_event(player, RequestPlayCard {})
                .await?
                .map(|ev| ev.card_idx as usize)
                .unwrap_or(0);
            let card_id = self.perform(PlayCard { player, card_index })?.unwrap();
            self.perform(ExecuteCard { player, card_id })?;
        }

        Ok(())
    }
}
