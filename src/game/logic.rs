use std::sync::Arc;

use anyhow::Result;

use crate::{
    game::{room::Room, state::*},
    grpc::RequestPlayCard,
};

impl Room {
    pub async fn main_loop(self: Arc<Self>) -> Result<()> {
        use PlayerId::{Player0, Player1};

        self.perform(Initalize);

        loop {
            self.turn(Player0).await?;
            self.turn(Player1).await?;

            if self.read_state(|gs| {
                gs.me(Player0).hand.is_empty()
                    && gs.me(Player1).hand.is_empty()
                    && gs.me(Player0).deck.is_empty()
                    && gs.me(Player1).deck.is_empty()
            }) {
                break;
            }

            self.perform(BumpRound);
        }

        self.perform(GameFinished);

        Ok(())
    }

    async fn turn(self: &Arc<Self>, player: PlayerId) -> Result<()> {
        self.perform(DrawCards { player, count: 1 });

        if self.read_state(|gs| gs.me(player).hand.len()) > 0 {
            // 如果玩家有手牌，则请求出牌
            let card_index = self
                .request_user_event(player, RequestPlayCard {})
                .await?
                .map(|ev| ev.card_idx as usize)
                .unwrap_or(0);
            let card_id = self.perform(PlayCard { player, card_index }).unwrap();
            self.perform(ExecuteCard { player, card_id });
        }

        Ok(())
    }
}
