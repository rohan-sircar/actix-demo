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
        match ws.next().await {
            Some(Ok(Frame::Text(txt))) => {
                let server_msg = serde_json::from_str::<WsServerEvent>(
                    std::str::from_utf8(&txt)?,
                )?;

                Ok(server_msg)
            }
            Some(Ok(frm)) => {
                Err(anyhow!("received wrong message frame: {frm:?}"))
            }
            Some(Err(err)) => Err(anyhow!("could not get ws message: {err:?}")),
            None => Err(anyhow!("could not get ws message")),
        }
    }
}

#[cfg(test)]
mod tests {

    use std::time::Duration;

    use crate::common::{sleep_bin_file, TestAppOptions};

    use super::*;
    use actix_demo::models::{
        misc::{Job, JobStatus},
        ws::MyProcessItem,
    };
    use actix_http::StatusCode;
    use actix_rt::time::sleep;
    use ws_utils::*;

    #[actix_rt::test]
    async fn send_message_test() {
        async {
            let connspec = common::pg_conn_string()?;
            let test_server =
                common::test_http_app(&connspec, TestAppOptions::default())
                    .await?;

            let addr = test_server.addr().to_string();
            // tracing::info!("Addr: {addr}");
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
        let res = async {
            let connspec = common::pg_conn_string()?;
            let test_server =
                common::test_http_app(&connspec, TestAppOptions::default())
                    .await?;

            let addr = test_server.addr().to_string();
            let client = Client::new();
            let username = common::DEFAULT_USER;
            let password = common::DEFAULT_USER;
            let token = get_token(&addr, username, password, &client).await?;
            let (_resp, mut ws) = connect_ws(&addr, &token, &client).await?;

            let mut resp = client
                .post(format!("http://{addr}/api/cmd"))
                .append_header((header::CONTENT_TYPE, "application/json"))
                .append_header((
                    header::AUTHORIZATION,
                    format!("Bearer {token}"),
                ))
                .send_body(r#"{"args":["arg1", "arg2"]}"#)
                .await
                .map_err(|err| anyhow!("{err}"))?;
            let job_resp = resp.json::<Job>().await?;
            let job_id = job_resp.job_id.to_string();
            assert_eq!(job_resp.started_by.as_str(), common::DEFAULT_USER);
            assert_eq!(job_resp.status, JobStatus::Pending);

            ws.send(ws_msg(&WsClientEvent::SubscribeJob {
                job_id: job_id.clone(),
            }))
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

            let msg = ws_take_one(&mut ws).await?;

            // sleep(Duration::from_millis(500)).await;

            if let WsServerEvent::CommandMessage {
                message: MyProcessItem::Done { code },
            } = msg
            {
                assert_eq!(&code, "0");
            } else {
                panic!("error wrong message type");
            };

            let mut resp = client
                .get(format!("http://{addr}/api/cmd/{job_id}"))
                .append_header((header::CONTENT_TYPE, "application/json"))
                .append_header((
                    header::AUTHORIZATION,
                    format!("Bearer {token}"),
                ))
                .send()
                .await
                .map_err(|err| anyhow!("{err}"))?;
            let job_resp = resp.json::<Job>().await?;
            assert_eq!(job_resp.started_by.as_str(), common::DEFAULT_USER);
            assert_eq!(job_resp.status, JobStatus::Completed);
            Ok::<(), anyhow::Error>(())
        }
        .await;

        tracing::info!("{res:?}");
        res.unwrap();
    }

    #[actix_rt::test]
    async fn abort_job_test() {
        let res = async {
            let connspec = common::pg_conn_string()?;
            let file = sleep_bin_file();
            let options = TestAppOptionsBuilder::default()
                .bin_file(file)
                .build()
                .unwrap();
            let test_server = common::test_http_app(&connspec, options).await?;

            let addr = test_server.addr().to_string();
            let client = Client::new();
            let username = common::DEFAULT_USER;
            let password = common::DEFAULT_USER;
            let token = get_token(&addr, username, password, &client).await?;
            let (_resp, mut ws) = connect_ws(&addr, &token, &client).await?;

            let mut resp = client
                .post(format!("http://{addr}/api/cmd"))
                .append_header((header::CONTENT_TYPE, "application/json"))
                .append_header((
                    header::AUTHORIZATION,
                    format!("Bearer {token}"),
                ))
                .send_body(r#"{"args":[]}"#)
                .await
                .map_err(|err| anyhow!("{err}"))?;
            let job_resp = resp.json::<Job>().await?;
            let job_id = job_resp.job_id.to_string();
            assert_eq!(job_resp.started_by.as_str(), common::DEFAULT_USER);
            assert_eq!(job_resp.status, JobStatus::Pending);

            ws.send(ws_msg(&WsClientEvent::SubscribeJob {
                job_id: job_id.clone(),
            }))
            .await?;

            let _ = ws_take_one(&mut ws).await?;

            sleep(Duration::from_millis(100)).await;

            let resp = client
                .delete(format!("http://{addr}/api/cmd/{job_id}"))
                .append_header((
                    header::AUTHORIZATION,
                    format!("Bearer {token}"),
                ))
                .send()
                .await
                .map_err(|err| anyhow!("{err}"))?;

            assert_eq!(resp.status(), StatusCode::OK);

            sleep(Duration::from_millis(500)).await;

            let mut resp = client
                .get(format!("http://{addr}/api/cmd/{job_id}"))
                .append_header((
                    header::AUTHORIZATION,
                    format!("Bearer {token}"),
                ))
                .send()
                .await
                .map_err(|err| anyhow!("{err}"))?;
            let job_resp = resp.json::<Job>().await?;
            assert_eq!(job_resp.started_by.as_str(), common::DEFAULT_USER);
            assert_eq!(job_resp.status, JobStatus::Aborted);
            Ok::<(), anyhow::Error>(())
        }
        .await;

        tracing::info!("{res:?}");
        res.unwrap();
    }
}
