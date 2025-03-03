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
use common::TestAppOptionsBuilder;

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

    use crate::common::{sleep_bin_file, TestAppOptions, WithToken};

    use super::*;
    use actix_demo::models::{
        misc::{Job, JobStatus},
        ws::MyProcessItem,
    };
    use actix_http::StatusCode;
    use actix_rt::time::sleep;
    use ws_utils::*;

    #[ignore]
    #[actix_rt::test]
    async fn send_message_test() {
        async {
            let (pg_connstr, _pg) = common::test_with_postgres().await?;
            let (redis_connstr, _redis) = common::test_with_redis().await?;
            let test_server = common::test_http_app(
                &pg_connstr,
                &redis_connstr,
                TestAppOptions::default(),
            )
            .await?;

            let addr = test_server.addr().to_string();
            // tracing::info!("Addr: {addr}");
            let client = Client::new();
            // let resp = test_server.get("/users").send().await;
            let username = common::DEFAULT_USER;
            let password = common::DEFAULT_USER;
            let token =
                common::get_http_token(&addr, username, password, &client)
                    .await?;
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

    #[ignore]
    #[actix_rt::test]
    async fn run_job_test() {
        let res: anyhow::Result<()> = async {
            let (pg_connstr, _pg) = common::test_with_postgres().await?;
            let (redis_connstr, _redis) = common::test_with_redis().await?;
            let test_server = common::test_http_app(
                &pg_connstr,
                &redis_connstr,
                TestAppOptions::default(),
            )
            .await?;

            let addr = test_server.addr().to_string();
            let client = Client::new();
            let username = common::DEFAULT_USER;
            let password = common::DEFAULT_USER;
            let token =
                common::get_http_token(&addr, username, password, &client)
                    .await?;

            let _ = tracing::info!("Connecting to WebSocket...");
            let (_resp, mut ws) = connect_ws(&addr, &token, &client).await?;
            let _ = tracing::info!("Successfully connected to WebSocket.");

            let mut resp = client
                .post(format!("http://{addr}/api/cmd"))
                .append_header((header::CONTENT_TYPE, "application/json"))
                .with_token(&token)
                .send_body(r#"{"args":["arg1", "arg2"]}"#)
                .await
                .map_err(|err| anyhow!("{err}"))?;
            let job_resp = resp.json::<Job>().await?;
            let job_id = job_resp.job_id;
            assert_eq!(job_resp.started_by.as_str(), common::DEFAULT_USER);
            assert_eq!(job_resp.status, JobStatus::Pending);

            let _ = tracing::info!(
                "Sending SubscribeJob message with job_id: {}",
                job_id
            );
            ws.send(ws_msg(&WsClientEvent::SubscribeJob { job_id }))
                .await?;
            let _ = tracing::info!("Finished sending SubscribeJob message.");

            sleep(Duration::from_millis(100)).await;

            let _ = tracing::info!("Waiting for first message...");
            let msg = ws_take_one(&mut ws).await?;
            let _ = tracing::info!("Received first message: {msg:?}");

            let _ = tracing::info!("Waiting for second message...");
            let msg = ws_take_one(&mut ws).await?;
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

            let msg = ws_take_one(&mut ws).await?;

            let _ = tracing::info!("Received message: {msg:?}");

            if let WsServerEvent::CommandMessage {
                message: MyProcessItem::Done { code },
            } = msg
            {
                assert_eq!(&code, "0");
            } else {
                panic!("error wrong message type");
            };

            let _ = tracing::info!(
                "Verifying that job status was set to completed"
            );

            let mut resp = client
                .get(format!("http://{addr}/api/cmd/{job_id}"))
                .append_header((header::CONTENT_TYPE, "application/json"))
                .with_token(&token)
                .send()
                .await
                .map_err(|err| anyhow!("{err}"))?;
            let job_resp = resp.json::<Job>().await?;
            assert_eq!(job_resp.started_by.as_str(), common::DEFAULT_USER);
            assert_eq!(job_resp.status, JobStatus::Completed);

            let _ =
                tracing::info!("Verified that job status was set to completed");

            Ok(())
        }
        .await;

        tracing::info!("{res:?}");
        res.unwrap();
    }

    #[ignore]
    #[actix_rt::test]
    async fn abort_job_test() {
        let res: anyhow::Result<()> = async {
            let (pg_connstr, _pg) = common::test_with_postgres().await?;
            let (redis_connstr, _redis) = common::test_with_redis().await?;
            let file = sleep_bin_file();
            let options = TestAppOptionsBuilder::default()
                .bin_file(file)
                .build()
                .unwrap();
            let test_server =
                common::test_http_app(&pg_connstr, &redis_connstr, options)
                    .await?;

            let addr = test_server.addr().to_string();
            let client = Client::new();
            let username = common::DEFAULT_USER;
            let password = common::DEFAULT_USER;
            let token =
                common::get_http_token(&addr, username, password, &client)
                    .await?;
            let (_resp, mut ws) = connect_ws(&addr, &token, &client).await?;

            let mut resp = client
                .post(format!("http://{addr}/api/cmd"))
                .append_header((header::CONTENT_TYPE, "application/json"))
                .with_token(&token)
                .send_body(r#"{"args":[]}"#)
                .await
                .map_err(|err| anyhow!("{err}"))?;
            let job_resp = resp.json::<Job>().await?;
            let job_id = job_resp.job_id;
            assert_eq!(job_resp.started_by.as_str(), common::DEFAULT_USER);
            assert_eq!(job_resp.status, JobStatus::Pending);

            ws.send(ws_msg(&WsClientEvent::SubscribeJob { job_id }))
                .await?;

            let _ = ws_take_one(&mut ws).await?;

            sleep(Duration::from_millis(100)).await;

            let resp = client
                .delete(format!("http://{addr}/api/cmd/{job_id}"))
                .with_token(&token)
                .send()
                .await
                .map_err(|err| anyhow!("{err}"))?;

            assert_eq!(resp.status(), StatusCode::OK);

            sleep(Duration::from_millis(500)).await;

            let mut resp = client
                .get(format!("http://{addr}/api/cmd/{job_id}"))
                .with_token(&token)
                .send()
                .await
                .map_err(|err| anyhow!("{err}"))?;
            let job_resp = resp.json::<Job>().await?;
            assert_eq!(job_resp.started_by.as_str(), common::DEFAULT_USER);
            assert_eq!(job_resp.status, JobStatus::Aborted);
            Ok(())
        }
        .await;

        tracing::info!("{res:?}");
        res.unwrap();
    }
}
