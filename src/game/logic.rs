use std::sync::Arc;

use anyhow::Result;

use crate::{
    game::{action::*, card::*, player::*, room::*, state::*, user::*},
    grpc::RequestTurnAction,
    system::{Entity, Query, exact, has},
};

impl Room {
    pub async fn main_loop(self: Arc<Self>) -> Result<()> {
        use PlayerId::{Player0, Player1};

        self.perform(Initalize);

        loop {
            self.turn(Player0).await?;
            self.turn(Player1).await?;

            if self.read(|world| {
                world.query(has::<InHand>()).count() == 0
                    && world.query(has::<InDeck>()).count() == 0
            }) {
                break;
            }

            self.perform(BumpRound);
        }

        self.perform(GameFinished);

        Ok(())
    }

    async fn turn(self: &Arc<Self>, player: PlayerId) -> Result<()> {
        self.perform(StartTurn { player });
        self.perform(DrawCards { player, count: 1 });

        while self.read(|world| {
            world
                .resource::<TurnTimer>()
                .map(|timer| !timer.0.remaining().is_zero())
                == Some(true)
        }) {
            let playable_cards = self.read(|world| {
                world
                    .query(has::<CardId>().and(exact(InHand(player))))
                    .map(|(e, _)| e.id())
                    .collect::<Vec<_>>()
            });
            let action = self
                .request_user_event(player, RequestTurnAction { playable_cards })
                .await?
                .unwrap_or(TurnAction::EndTurn(Default::default()));
            match action {
                TurnAction::PlayCard(play_card) => {
                    let card = Entity::from(play_card.entity);
                    let Some(card_id) = self.read(|world| card.get::<CardId>(world).copied())
                    else {
                        continue; // 如果获取卡牌 ID 失败，继续等待
                    };
                    self.perform(PlayCard { player, card });
                    self.perform(ExecuteCard { player, card_id });
                }
                TurnAction::EndTurn(_) => break,
            }
        }

        self.perform(EndTurn { player });
        Ok(())
    }
}
