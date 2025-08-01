use std::sync::Arc;

use anyhow::Result;

use crate::{
    card::REGISTRY,
    game::{action::*, card::*, player::*, room::*, state::*, user::*},
    grpc::{Cost, CostProvider, RequestCostAction, RequestTurnAction},
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

        'turn: while self.read(|world| {
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
                .await?;
            match action {
                Some(TurnAction::PlayCard(play_card)) => {
                    let card = Entity::from(play_card.entity);
                    if let Some(card_id) = self.read(|world| card.get::<CardId>(world).copied())
                        && let Some(prototype) = REGISTRY.cards.get(&card_id)
                        && let Some(cost) = prototype.cost()
                    {
                        if cost > 0 {
                            let providers = self.read(|world| {
                                world
                                    .query(exact(Faith(player)))
                                    .map(|(e, _)| CostProvider {
                                        entity: e.id(),
                                        provided: Some(Cost { any: 1 }),
                                    })
                                    .collect::<Vec<_>>()
                            });
                            let cost = Some(Cost { any: cost });
                            let Some(_r) = self
                                .request_user_event(player, RequestCostAction { cost, providers })
                                .await?
                            else {
                                break 'turn;
                            };
                        }

                        self.perform(PlayCard { player, card });
                        self.perform(ExecuteCard { player, card_id });
                    }
                }
                Some(TurnAction::EndTurn(_)) | None => break 'turn,
            }
        }

        self.perform(EndTurn { player });
        Ok(())
    }
}
