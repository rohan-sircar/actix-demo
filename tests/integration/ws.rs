use futures::prelude::*;
use std::str::FromStr;

use crate::common;

use actix_codec::Framed;
use actix_demo::models::users::UserId;
use actix_demo::models::ws::{WsClientEvent, WsServerEvent};
use actix_demo::utils;
use actix_http::header;
use actix_http::ws::{Codec, Frame};
use actix_ws::Message;
use anyhow::anyhow;
use awc::{BoxedSocket, Client, ClientResponse};
use bytestring::ByteString;

pub mod ws_utils {
    use super::*;
    pub type WsClient = Framed<BoxedSocket, Codec>;

    pub async fn get_token(
        addr: &str,
        username: &str,
        password: &str,
        client: &Client,
    ) -> anyhow::Result<String> {
        let resp = client
            .post(format!("http://{addr}/api/login"))
            .insert_header((header::CONTENT_TYPE, "application/json"))
            .send_body(format!(
                r#"{{"username":"{username}","password":"{password}"}}"#
            ))
            .await
            .map_err(|err| anyhow!("{err}"))?;
        let token = resp
            .headers()
            .get("X-AUTH-TOKEN")
            .unwrap()
            .to_str()?
            .to_owned();
        Ok(token)
    }

    pub async fn _create_user(
        addr: &str,
        username: &str,
        password: &str,
        client: &Client,
    ) -> anyhow::Result<()> {
        let _ = client
            .post(format!("http://{addr}/api/registration"))
            .insert_header(("content-type", "application/json"))
            .send_body(format!(
                r#"{{"username":"{username}","password":"{password}"}}"#
            ))
            .await
            .map_err(|err| anyhow!("{err}"))?;

        Ok(())
    }

    pub fn ws_msg(msg: &WsClientEvent) -> Message {
        Message::Text(ByteString::from(utils::jstr(msg)))
    }

    pub async fn connect_ws(
        addr: &str,
        token: &str,
        client: &Client,
    ) -> anyhow::Result<(ClientResponse, WsClient)> {
        client
            .ws(format!("http://{addr}/ws?token={token}"))
            .connect()
            .await
            .map_err(|err| anyhow!("{err}"))
    }

    pub async fn ws_take_one(
        ws: &mut WsClient,
    ) -> anyhow::Result<WsServerEvent> {
        if let Some(Ok(Frame::Text(txt))) = ws.next().await {
            let server_msg = serde_json::from_str::<WsServerEvent>(
                std::str::from_utf8(&txt)?,
            )?;

            Ok(server_msg)
        } else {
            Err(anyhow!("could not get ws message"))
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use actix_demo::models::ws::MyProcessItem;
    use ws_utils::*;

    #[actix_rt::test]
    async fn send_message_test() {
        async {
            let connspec = common::pg_conn_string()?;
            let test_server = common::test_http_app(&connspec).await?;

            let addr = test_server.addr().to_string();
            tracing::info!("Addr: {addr}");
            let client = Client::new();
            // let resp = test_server.get("/users").send().await;
            let username = common::DEFAULT_USER;
            let password = common::DEFAULT_USER;
            let token = get_token(&addr, username, password, &client).await?;
            let (_resp, mut ws) = connect_ws(&addr, &token, &client).await?;

            ws.send(ws_msg(&WsClientEvent::SendMessage {
                receiver: UserId::from_str("1").unwrap(),
                message: "hello".to_owned(),
            }))
            .await?;

            let msg = ws_take_one(&mut ws).await?;

            if let WsServerEvent::SentMessage {
                id: _,
                sender,
                message,
            } = msg
            {
                assert_eq!(sender.as_uint(), 1);
                assert_eq!(&message, "hello");
            } else {
                panic!("error wrong message type");
            };
            Ok::<(), anyhow::Error>(())
        }
        .await
        .unwrap()
    }

    #[actix_rt::test]
    async fn run_job_test() {
        async {
            let connspec = common::pg_conn_string()?;
            let test_server = common::test_http_app(&connspec).await?;

            let addr = test_server.addr().to_string();
            tracing::info!("Addr: {addr}");
            let client = Client::new();
            let username = common::DEFAULT_USER;
            let password = common::DEFAULT_USER;
            let token = get_token(&addr, username, password, &client).await?;
            let (_resp, mut ws) = connect_ws(&addr, &token, &client).await?;

            let mut resp = client
                .post(format!("http://{addr}/api/public/cmd"))
                .append_header((header::CONTENT_TYPE, "application/json"))
                .send_body(r#"{"args":["arg1", "arg2"]}"#)
                .await
                .map_err(|err| anyhow!("{err}"))?;
            let job_id = std::str::from_utf8(&resp.body().await?)?.to_owned();

            ws.send(ws_msg(&WsClientEvent::SubscribeJob { job_id }))
                .await?;

            let msg = ws_take_one(&mut ws).await?;

            if let WsServerEvent::CommandMessage {
                message: MyProcessItem::Line { value },
            } = msg
            {
                assert_eq!(&value, "hello world arg1 arg2");
            } else {
                panic!("error wrong message type");
            };
            Ok::<(), anyhow::Error>(())
        }
        .await
        .unwrap()
    }
}