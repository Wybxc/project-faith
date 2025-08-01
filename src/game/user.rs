#![allow(unreachable_patterns)]

use crate::grpc::*;

pub trait UserEvent {
    type Response;

    fn into_rpc(self) -> request_user_event::EventType;
    fn from_rpc(response: user_event::EventType) -> anyhow::Result<Self::Response>;
}

pub enum TurnAction {
    PlayCard(PlayCard),
    EndTurn(EndTurn),
}

impl UserEvent for RequestTurnAction {
    type Response = TurnAction;

    fn into_rpc(self) -> request_user_event::EventType {
        request_user_event::EventType::TurnAction(self)
    }

    fn from_rpc(response: user_event::EventType) -> anyhow::Result<Self::Response> {
        match response {
            user_event::EventType::PlayCard(ev) => Ok(TurnAction::PlayCard(ev)),
            user_event::EventType::EndTurn(ev) => Ok(TurnAction::EndTurn(ev)),
            _ => Err(anyhow::anyhow!("Invalid event type for RequestTurnAction")),
        }
    }
}

impl UserEvent for RequestCostAction {
    type Response = PayCost;

    fn into_rpc(self) -> request_user_event::EventType {
        request_user_event::EventType::CostAction(self)
    }

    fn from_rpc(response: user_event::EventType) -> anyhow::Result<Self::Response> {
        match response {
            user_event::EventType::PayCost(ev) => Ok(ev),
            _ => Err(anyhow::anyhow!("Invalid event type for RequestCostAction")),
        }
    }
}
