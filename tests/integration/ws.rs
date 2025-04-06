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
    use awc::cookie::Cookie;

    use super::*;
    pub type WsClient = Framed<BoxedSocket, Codec>;

    pub fn ws_msg(msg: &WsClientEvent) -> Message {
        Message::Text(ByteString::from(utils::jstr(msg)))
    }

    pub async fn connect_ws(
        addr: &str,
        token: &str,
        client: &Client,
    ) -> anyhow::Result<(ClientResponse, WsClient)> {
        client
            .ws(format!("http://{addr}/ws"))
            .cookie(Cookie::new("X-AUTH-TOKEN", token))
            .connect()
            .await
            .map_err(|err| anyhow!("{err}"))
    }

    pub async fn ws_take_one(
        ws: &mut WsClient,
    ) -> anyhow::Result<WsServerEvent> {
        loop {
            match ws.next().await {
                Some(Ok(Frame::Text(txt))) => {
                    let server_msg = serde_json::from_str::<WsServerEvent>(
                        std::str::from_utf8(&txt)?,
                    )?;
                    return Ok(server_msg);
                }
                Some(Ok(Frame::Ping(_))) => {
                    // Ignore ping frames from heartbeat
                    // add log message here
                    tracing::info!("Discarding ping message");
                    continue;
                }
                Some(Ok(frm)) => {
                    return Err(anyhow!(
                        "received wrong message frame: {frm:?}"
                    ));
                }
                Some(Err(err)) => {
                    return Err(anyhow!("could not get ws message: {err:?}"))
                }
                None => return Err(anyhow!("could not get ws message")),
            }
        }
    }
}

#[cfg(test)]
mod tests {

    use std::time::Duration;

    use crate::common::WithToken;

    use super::*;
    use actix_demo::models::{
        misc::{Job, JobStatus},
        ws::MyProcessItem,
    };
    use actix_http::StatusCode;
    use actix_rt::time::sleep;
    use jwt_simple::prelude::HS256Key;
    use ws_utils::*;

    #[ignore]
    #[actix_rt::test]
    async fn send_message_test() {
        let ctx = common::TestContext::new(None).await;
        let username = common::DEFAULT_USER;
        let password = common::DEFAULT_USER;
        let token =
            common::get_http_token(&ctx.addr, username, password, &ctx.client)
                .await
                .unwrap();

        let (_resp, mut ws) =
            connect_ws(&ctx.addr, &token, &ctx.client).await.unwrap();

        ws.send(ws_msg(&WsClientEvent::SendMessage {
            receiver: UserId::from_str("1").unwrap(),
            message: "hello".to_owned(),
        }))
        .await
        .unwrap();

        let msg = ws_take_one(&mut ws).await.unwrap();

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
    }

    #[ignore]
    #[actix_rt::test]
    async fn run_job_test() {
        let ctx = common::TestContext::new(None).await;
        let username = common::DEFAULT_USER;
        let password = common::DEFAULT_USER;
        let token =
            common::get_http_token(&ctx.addr, username, password, &ctx.client)
                .await
                .unwrap();
        let jwt_key = HS256Key::from_bytes("test".as_bytes());

        let claims = utils::get_claims(&jwt_key, &token).unwrap();
        let user_id = claims.custom.user_id;

        let _ = tracing::info!("Connecting to WebSocket...");
        let (_resp, mut ws) =
            connect_ws(&ctx.addr, &token, &ctx.client).await.unwrap();
        let _ = tracing::info!("Successfully connected to WebSocket.");

        let mut resp = ctx
            .test_server
            .post("/api/cmd")
            .append_header((header::CONTENT_TYPE, "application/json"))
            .with_token(&token)
            .send_body(r#"{"args":["arg1", "arg2"]}"#)
            .await
            .unwrap();
        let job_resp = resp.json::<Job>().await.unwrap();
        let job_id = job_resp.job_id;
        assert_eq!(job_resp.started_by, user_id);
        assert_eq!(job_resp.status, JobStatus::Pending);

        let _ = tracing::info!(
            "Sending SubscribeJob message with job_id: {}",
            job_id
        );
        ws.send(ws_msg(&WsClientEvent::SubscribeJob { job_id }))
            .await
            .unwrap();
        let _ = tracing::info!("Finished sending SubscribeJob message.");

        sleep(Duration::from_millis(100)).await;

        let _ = tracing::info!("Waiting for first message...");
        let msg = ws_take_one(&mut ws).await.unwrap();
        let _ = tracing::info!("Received first message: {msg:?}");

        let _ = tracing::info!("Waiting for second message...");
        let msg = ws_take_one(&mut ws).await.unwrap();
        let _ = tracing::info!("Received second message: {msg:?}");

        if let WsServerEvent::CommandMessage {
            message: MyProcessItem::Line { value },
        } = msg
        {
            assert_eq!(&value, "hello world arg1 arg2");
        } else {
            panic!("error wrong message type");
        };

        sleep(Duration::from_millis(100)).await;

        let msg = ws_take_one(&mut ws).await.unwrap();

        let _ = tracing::info!("Received message: {msg:?}");

        if let WsServerEvent::CommandMessage {
            message: MyProcessItem::Done { code },
        } = msg
        {
            assert_eq!(&code, "0");
        } else {
            panic!("error wrong message type");
        };

        let _ =
            tracing::info!("Verifying that job status was set to completed");

        let mut resp = ctx
            .test_server
            .get(format!("/api/cmd/{job_id}"))
            .append_header((header::CONTENT_TYPE, "application/json"))
            .with_token(&token)
            .send()
            .await
            .unwrap();
        let job_resp = resp.json::<Job>().await.unwrap();
        assert_eq!(job_resp.started_by, user_id);
        assert_eq!(job_resp.status, JobStatus::Completed);

        let _ = tracing::info!("Verified that job status was set to completed");
    }

    #[ignore]
    #[actix_rt::test]
    async fn abort_job_test() {
        let file = common::sleep_bin_file();
        let options = common::TestAppOptionsBuilder::default()
            .bin_file(file)
            .build()
            .unwrap();
        let ctx = common::TestContext::new(Some(options)).await;
        let username = common::DEFAULT_USER;
        let password = common::DEFAULT_USER;
        let token =
            common::get_http_token(&ctx.addr, username, password, &ctx.client)
                .await
                .unwrap();
        let jwt_key = HS256Key::from_bytes("test".as_bytes());

        let claims = utils::get_claims(&jwt_key, &token).unwrap();
        let user_id = claims.custom.user_id;
        let (_resp, mut ws) =
            connect_ws(&ctx.addr, &token, &ctx.client).await.unwrap();

        let mut resp = ctx
            .test_server
            .post("/api/cmd")
            .append_header((header::CONTENT_TYPE, "application/json"))
            .with_token(&token)
            .send_body(r#"{"args":[]}"#)
            .await
            .unwrap();
        let job_resp = resp.json::<Job>().await.unwrap();
        let job_id = job_resp.job_id;
        assert_eq!(job_resp.started_by, user_id);
        assert_eq!(job_resp.status, JobStatus::Pending);

        ws.send(ws_msg(&WsClientEvent::SubscribeJob { job_id }))
            .await
            .unwrap();

        let _ = ws_take_one(&mut ws).await.unwrap();

        sleep(Duration::from_millis(100)).await;

        let resp = ctx
            .test_server
            .delete(format!("/api/cmd/{job_id}"))
            .with_token(&token)
            .send()
            .await
            .unwrap();

        assert_eq!(resp.status(), StatusCode::OK);

        sleep(Duration::from_millis(500)).await;

        let mut resp = ctx
            .test_server
            .get(format!("/api/cmd/{job_id}"))
            .with_token(&token)
            .send()
            .await
            .unwrap();
        let job_resp = resp.json::<Job>().await.unwrap();
        assert_eq!(job_resp.started_by, user_id);
        assert_eq!(job_resp.status, JobStatus::Aborted);
    }

    #[ignore]
    #[actix_rt::test]
    async fn subscribe_job_invalid_job_id_test() {
        let ctx = common::TestContext::new(None).await;
        let username = common::DEFAULT_USER;
        let password = common::DEFAULT_USER;
        let token =
            common::get_http_token(&ctx.addr, username, password, &ctx.client)
                .await
                .unwrap();

        let (_resp, mut ws) =
            connect_ws(&ctx.addr, &token, &ctx.client).await.unwrap();
        let _ = tracing::info!("Connected to WebSocket");

        let invalid_job_id = uuid::Uuid::new_v4(); // Create a random UUID that doesn't exist
        let _ = tracing::info!("Generated invalid job ID: {}", invalid_job_id);

        ws.send(ws_msg(&WsClientEvent::SubscribeJob {
            job_id: invalid_job_id,
        }))
        .await
        .unwrap();
        let _ =
            tracing::info!("Sent SubscribeJob message with invalid job ID.");

        let msg = ws_take_one(&mut ws).await.unwrap();
        let _ = tracing::info!("Received message: {:?}", msg);

        if let WsServerEvent::Error { id: _, cause } = msg {
            assert!(cause.contains("Job with id:"));
            assert!(cause.contains("does not exist"));
            let _ =
                tracing::info!("Received expected error message: {}", cause);
        } else {
            panic!("error wrong message type: {:?}", msg);
        }
    }
}
