use std::sync::Arc;

use anyhow::Result;

use crate::{
    game::{
        card::{CardId, InHand},
        room::Room,
        state::*,
        user::TurnAction,
    },
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

            if self.read(|system| {
                let gs = system.resource::<GameState>().unwrap();
                gs.me(Player0).deck.is_empty() && gs.me(Player1).deck.is_empty()
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

        while self.read(|system| {
            system
                .resource::<TurnTimer>()
                .map(|timer| !timer.0.remaining().is_zero())
                == Some(true)
        }) {
            let playable_cards = self.read(|system| {
                system
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
                    self.perform(PlayCard { player, card });
                    let Some(card_id) = self.read(|system| card.get::<CardId>(system).copied())
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
