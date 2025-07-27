use std::sync::Arc;

use anyhow::Result;

use crate::{
    game::{card::CardId, room::Room, state::*, user::TurnAction},
    grpc::RequestTurnAction,
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
        self.perform(TurnStart { player });
        self.perform(DrawCards { player, count: 1 });

        while !self.read_state(|gs| gs.turn_time_remaining().is_zero()) {
            let action = self
                .request_user_event(player, RequestTurnAction {})
                .await?
                .unwrap_or(TurnAction::EndTurn(Default::default()));
            match action {
                TurnAction::PlayCard(play_card) => {
                    let card_index = play_card.card_idx as usize;
                    let Some(card) = self.perform(PlayCard { player, card_index }) else {
                        continue; // 如果出牌失败，继续等待
                    };
                    let Some(card_id) =
                        self.read_state(|gs| gs.system().get::<CardId>(card).copied())
                    else {
                        continue; // 如果获取卡牌 ID 失败，继续等待
                    };
                    self.perform(ExecuteCard { player, card_id });
                }
                TurnAction::EndTurn(_) => {
                    self.perform(EndTurn { player });
                    break;
                }
            }
        }

        Ok(())
    }
}
