use crate::models::users::UserUuid;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind")]
pub enum MyProcessItem {
    Line { value: String },
    Error { cause: String },
    Done { code: String },
}

#[derive(Clone, Serialize, Deserialize, Debug)]
#[serde(tag = "kind")]
pub enum WsClientEvent {
    SendMessage {
        receiver: UserUuid,
        message: String,
    },
    #[serde(rename_all = "camelCase")]
    SubscribeJob {
        job_id: uuid::Uuid,
    },
    Error {
        cause: String,
    },
}

#[derive(Clone, Serialize, Deserialize, Debug)]

pub struct SentMessage {
    pub sender: UserUuid,
    pub message: String,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
#[serde(tag = "kind")]
pub enum WsServerEvent {
    SentMessage {
        id: String,
        sender: UserUuid,
        message: String,
    },
    CommandMessage {
        message: MyProcessItem,
    },
    Error {
        id: Option<String>,
        cause: String,
    },
}
