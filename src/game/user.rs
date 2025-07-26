#![allow(unreachable_patterns)]

use crate::grpc::*;

pub trait UserEvent {
    type Response;

    fn into_rpc(self) -> request_user_event::EventType;
    fn from_rpc(response: user_event::EventType) -> anyhow::Result<Self::Response>;
}

macro_rules! impl_user_event {
    ($name:ident : $event:ident => $response:ident) => {
        impl UserEvent for $event {
            type Response = $response;

            fn into_rpc(self) -> request_user_event::EventType {
                request_user_event::EventType::$name(self)
            }

            fn from_rpc(response: user_event::EventType) -> anyhow::Result<Self::Response> {
                match response {
                    user_event::EventType::$name(ev) => Ok(ev),
                    _ => Err(anyhow::anyhow!(concat!(
                        "Invalid event type for ",
                        stringify!($event)
                    ))),
                }
            }
        }
    };
}

impl_user_event!(PlayCard: RequestPlayCard => PlayCard);
