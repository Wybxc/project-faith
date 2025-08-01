use std::sync::LazyLock;
use tonic::{Request, Response, Status, async_trait};

use crate::game::card::{CardId, Prototype, Registry, draw_cards};
use crate::grpc::*;

pub struct Card;

#[async_trait]
impl card_service_server::CardService for Card {
    async fn get_card_prototypes(
        &self,
        _request: Request<GetCardPrototypesRequest>,
    ) -> Result<Response<GetCardPrototypesResponse>, Status> {
        static PROTOTYPES: LazyLock<GetCardPrototypesResponse> = LazyLock::new(|| {
            let mut response = GetCardPrototypesResponse::default();
            for (card_id, prototype) in REGISTRY.cards.iter() {
                let card_proto = match prototype {
                    Prototype::Order(order) => CardPrototype {
                        name: order.name.clone(),
                        description: order.description.clone(),
                    },
                    Prototype::Faith(faith) => CardPrototype {
                        name: faith.name.clone(),
                        description: faith.description.clone(),
                    },
                };
                response.prototypes.insert(card_id.0, card_proto);
            }
            response
        });
        Ok(Response::new(PROTOTYPES.clone()))
    }
}

pub static REGISTRY: LazyLock<Registry> = LazyLock::new(|| {
    let mut registry = Registry::new();

    registry.order(|builder| {
        builder
            .card_id(CardId(7001))
            .name("测试卡7001")
            .description("抽一张牌。")
            .cost(0)
            .skill(draw_cards(1))
            .build()
    });
    registry.order(|builder| {
        builder
            .card_id(CardId(7002))
            .name("测试卡7002")
            .description("抽两张牌。")
            .cost(1)
            .skill(draw_cards(2))
            .build()
    });

    registry.faith(|builder| {
        builder
            .card_id(CardId(8001))
            .name("本色")
            .description("横置以支付1点无色信念。")
            .build()
    });

    registry
});
