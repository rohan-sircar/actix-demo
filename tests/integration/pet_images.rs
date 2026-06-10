use actix_web::http::StatusCode;
use awc::Client;
use image::{DynamicImage, ImageFormat};
use std::io::Cursor;

use crate::common::{register_and_login, TestContext, WithToken};

/// Generate a valid PNG image bytes of the given dimensions
fn make_test_image(width: u32, height: u32, color: [u8; 3]) -> Vec<u8> {
    let img = image::RgbImage::from_pixel(width, height, image::Rgb(color));
    let mut buf = Vec::new();
    img.write_to(&mut Cursor::new(&mut buf), ImageFormat::Png)
        .unwrap();
    buf
}

/// Generate a valid WebP image bytes of the given dimensions
fn make_test_webp(width: u32, height: u32, color: [u8; 3]) -> Vec<u8> {
    let img = DynamicImage::ImageRgba8(image::RgbaImage::from_pixel(
        width,
        height,
        image::Rgba([color[0], color[1], color[2], 255]),
    ));
    let mut buf = Vec::new();
    img.write_to(&mut Cursor::new(&mut buf), ImageFormat::WebP)
        .unwrap();
    buf
}

/// Send a raw binary POST request and return status + response body
async fn send_upload_request(
    client: &Client,
    addr: &str,
    token: &str,
    pet_uuid: &str,
    image_bytes: Vec<u8>,
    content_type: &str,
) -> (StatusCode, serde_json::Value) {
    let url = format!("http://{addr}/api/v1/user/pets/{}/images", pet_uuid);
    let mut resp = client
        .post(&url)
        .insert_header(("cookie", format!("X-AUTH-TOKEN={}", token)))
        .insert_header(("content-type", content_type))
        .send_body(image_bytes)
        .await
        .unwrap();
    let status = resp.status();
    let body = resp.json::<serde_json::Value>().await.unwrap();
    (status, body)
}

/// Create a pet and return its UUID
async fn create_pet(
    ctx: &TestContext,
    token: &str,
    name: &str,
    species: &str,
) -> String {
    let mut resp = ctx
        .test_server
        .post("/api/v1/user/pets")
        .with_token(token)
        .append_header((
            actix_web::http::header::CONTENT_TYPE,
            "application/json",
        ))
        .send_json(&serde_json::json!({
            "name": name,
            "species": species
        }))
        .await
        .unwrap();

    let body: serde_json::Value = resp.json().await.unwrap();
    body["pet_uuid"].as_str().unwrap().to_string()
}

mod pet_images_api {
    use super::*;

    #[actix_rt::test]
    async fn upload_image_success() {
        let ctx = TestContext::new(None).await;
        let token = register_and_login(&ctx, "imgowner1", "test123").await;

        let pet_uuid = create_pet(&ctx, &token, "Buddy", "dog").await;

        let image_bytes = make_test_image(800, 600, [255, 0, 0]);
        let (status, body) = send_upload_request(
            &ctx.client,
            &ctx.addr,
            &token,
            &pet_uuid,
            image_bytes,
            "image/png",
        )
        .await;

        assert_eq!(status, StatusCode::CREATED);
        assert_eq!(body["format"], "webp");
        assert_eq!(body["is_primary"], true);
        assert_eq!(body["sort_order"], 0);
        assert!(body["uuid"].as_str().is_some());
        assert!(body["id"].as_u64().is_some());
        assert!(body["created_at"].as_str().is_some());
    }

    #[actix_rt::test]
    async fn upload_image_auto_sets_primary() {
        let ctx = TestContext::new(None).await;
        let token = register_and_login(&ctx, "imgowner2", "test123").await;

        let pet_uuid = create_pet(&ctx, &token, "Buddy", "dog").await;

        // First upload — should be primary
        let image_bytes = make_test_image(800, 600, [255, 0, 0]);
        let (status, body) = send_upload_request(
            &ctx.client,
            &ctx.addr,
            &token,
            &pet_uuid,
            image_bytes,
            "image/png",
        )
        .await;
        assert_eq!(status, StatusCode::CREATED);
        assert_eq!(body["is_primary"], true);
        assert_eq!(body["sort_order"], 0);

        // Second upload — should NOT be primary
        let image_bytes2 = make_test_image(600, 800, [0, 255, 0]);
        let (status2, body2) = send_upload_request(
            &ctx.client,
            &ctx.addr,
            &token,
            &pet_uuid,
            image_bytes2,
            "image/png",
        )
        .await;
        assert_eq!(status2, StatusCode::CREATED);
        assert_eq!(body2["is_primary"], false);
        assert_eq!(body2["sort_order"], 1);
    }

    #[actix_rt::test]
    async fn upload_image_png_accepted() {
        let ctx = TestContext::new(None).await;
        let token = register_and_login(&ctx, "imgowner3", "test123").await;

        let pet_uuid = create_pet(&ctx, &token, "Max", "dog").await;

        let image_bytes = make_test_image(1024, 768, [0, 0, 255]);
        let (status, _) = send_upload_request(
            &ctx.client,
            &ctx.addr,
            &token,
            &pet_uuid,
            image_bytes,
            "image/png",
        )
        .await;

        assert_eq!(status, StatusCode::CREATED);
    }

    #[actix_rt::test]
    async fn upload_image_webp_accepted() {
        let ctx = TestContext::new(None).await;
        let token = register_and_login(&ctx, "imgowner4", "test123").await;

        let pet_uuid = create_pet(&ctx, &token, "Max", "dog").await;

        let image_bytes = make_test_webp(800, 600, [0, 0, 255]);
        let (status, _) = send_upload_request(
            &ctx.client,
            &ctx.addr,
            &token,
            &pet_uuid,
            image_bytes,
            "image/webp",
        )
        .await;

        assert_eq!(status, StatusCode::CREATED);
    }

    #[actix_rt::test]
    async fn upload_image_rejects_non_image() {
        let ctx = TestContext::new(None).await;
        let token = register_and_login(&ctx, "imgowner5", "test123").await;

        let pet_uuid = create_pet(&ctx, &token, "Buddy", "dog").await;

        let fake_bytes = b"not an image".to_vec();
        let (status, _) = send_upload_request(
            &ctx.client,
            &ctx.addr,
            &token,
            &pet_uuid,
            fake_bytes,
            "image/png",
        )
        .await;

        assert_eq!(status, StatusCode::BAD_REQUEST);
    }

    #[actix_rt::test]
    async fn upload_image_rejects_oversized() {
        let ctx = TestContext::new(None).await;
        let token = register_and_login(&ctx, "imgowner6", "test123").await;

        let pet_uuid = create_pet(&ctx, &token, "Buddy", "dog").await;

        // Generate a valid PNG larger than 5MB using a gradient pattern
        let width = 4096;
        let height = 4096;
        let mut img = image::RgbImage::new(width, height);
        for (x, y, pixel) in img.enumerate_pixels_mut() {
            pixel.0 = [
                (x % 256) as u8,
                (y % 256) as u8,
                ((x.wrapping_add(y)) % 256) as u8,
            ];
        }
        let mut buf = Vec::new();
        img.write_to(&mut std::io::Cursor::new(&mut buf), ImageFormat::Png)
            .unwrap();

        let url =
            format!("http://{}/api/v1/user/pets/{}/images", ctx.addr, pet_uuid);
        let resp = ctx
            .client
            .post(&url)
            .insert_header(("cookie", format!("X-AUTH-TOKEN={}", token)))
            .insert_header(("content-type", "image/png"))
            .send_body(buf)
            .await;

        // Server may close connection early for oversized payloads (broken pipe)
        // or return PAYLOAD_TOO_LARGE
        if let Ok(resp) = resp {
            assert_eq!(resp.status(), StatusCode::PAYLOAD_TOO_LARGE);
        }
    }

    #[actix_rt::test]
    async fn upload_image_ownership_enforcement() {
        let ctx = TestContext::new(None).await;
        let token_a = register_and_login(&ctx, "owner_a_img", "test123").await;
        let token_b = register_and_login(&ctx, "owner_b_img", "test123").await;

        let pet_uuid = create_pet(&ctx, &token_a, "Buddy", "dog").await;

        let image_bytes = make_test_image(800, 600, [255, 0, 0]);
        let (status, _) = send_upload_request(
            &ctx.client,
            &ctx.addr,
            &token_b,
            &pet_uuid,
            image_bytes,
            "image/png",
        )
        .await;

        assert_eq!(status, StatusCode::NOT_FOUND);
    }

    #[actix_rt::test]
    async fn list_images_success() {
        let ctx = TestContext::new(None).await;
        let token = register_and_login(&ctx, "imgowner7", "test123").await;

        let pet_uuid = create_pet(&ctx, &token, "Buddy", "dog").await;

        // Upload two images
        let image_bytes1 = make_test_image(800, 600, [255, 0, 0]);
        send_upload_request(
            &ctx.client,
            &ctx.addr,
            &token,
            &pet_uuid,
            image_bytes1,
            "image/png",
        )
        .await;

        let image_bytes2 = make_test_image(600, 800, [0, 255, 0]);
        send_upload_request(
            &ctx.client,
            &ctx.addr,
            &token,
            &pet_uuid,
            image_bytes2,
            "image/png",
        )
        .await;

        // List images
        let mut resp = ctx
            .test_server
            .get(format!("/api/v1/user/pets/{}/images", pet_uuid))
            .with_token(&token)
            .send()
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let images: Vec<serde_json::Value> = resp.json().await.unwrap();
        assert_eq!(images.len(), 2);
        // Verify sort_order is correct
        assert_eq!(images[0]["sort_order"], 0);
        assert_eq!(images[1]["sort_order"], 1);
    }

    #[actix_rt::test]
    async fn list_images_empty_for_pet() {
        let ctx = TestContext::new(None).await;
        let token = register_and_login(&ctx, "imgowner8", "test123").await;

        let pet_uuid = create_pet(&ctx, &token, "Buddy", "dog").await;

        let mut resp = ctx
            .test_server
            .get(format!("/api/v1/user/pets/{}/images", pet_uuid))
            .with_token(&token)
            .send()
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let images: Vec<serde_json::Value> = resp.json().await.unwrap();
        assert!(images.is_empty());
    }

    #[actix_rt::test]
    async fn set_primary_image_toggles() {
        let ctx = TestContext::new(None).await;
        let token = register_and_login(&ctx, "imgowner9", "test123").await;

        let pet_uuid = create_pet(&ctx, &token, "Buddy", "dog").await;

        // Upload two images
        let image_bytes1 = make_test_image(800, 600, [255, 0, 0]);
        let (_, body1) = send_upload_request(
            &ctx.client,
            &ctx.addr,
            &token,
            &pet_uuid,
            image_bytes1,
            "image/png",
        )
        .await;
        let image_uuid_1 = body1["uuid"].as_str().unwrap().to_string();

        let image_bytes2 = make_test_image(600, 800, [0, 255, 0]);
        let (_, body2) = send_upload_request(
            &ctx.client,
            &ctx.addr,
            &token,
            &pet_uuid,
            image_bytes2,
            "image/png",
        )
        .await;
        let image_uuid_2 = body2["uuid"].as_str().unwrap().to_string();

        // Set second image as primary
        let resp = ctx
            .test_server
            .patch(format!(
                "/api/v1/user/pets/{}/images/{}",
                pet_uuid, image_uuid_2
            ))
            .with_token(&token)
            .append_header((
                actix_web::http::header::CONTENT_TYPE,
                "application/json",
            ))
            .send_json(&serde_json::json!({"is_primary": true}))
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);

        // List images and verify toggle
        let mut resp = ctx
            .test_server
            .get(format!("/api/v1/user/pets/{}/images", pet_uuid))
            .with_token(&token)
            .send()
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let images: Vec<serde_json::Value> = resp.json().await.unwrap();
        let img1 = images.iter().find(|i| i["uuid"] == image_uuid_1).unwrap();
        let img2 = images.iter().find(|i| i["uuid"] == image_uuid_2).unwrap();
        assert_eq!(img1["is_primary"], false);
        assert_eq!(img2["is_primary"], true);
    }

    #[actix_rt::test]
    async fn set_primary_rejects_false() {
        let ctx = TestContext::new(None).await;
        let token = register_and_login(&ctx, "imgowner10", "test123").await;

        let pet_uuid = create_pet(&ctx, &token, "Buddy", "dog").await;

        let image_bytes = make_test_image(800, 600, [255, 0, 0]);
        let (_, body) = send_upload_request(
            &ctx.client,
            &ctx.addr,
            &token,
            &pet_uuid,
            image_bytes,
            "image/png",
        )
        .await;
        let image_uuid = body["uuid"].as_str().unwrap().to_string();

        let resp = ctx
            .test_server
            .patch(format!(
                "/api/v1/user/pets/{}/images/{}",
                pet_uuid, image_uuid
            ))
            .with_token(&token)
            .append_header((
                actix_web::http::header::CONTENT_TYPE,
                "application/json",
            ))
            .send_json(&serde_json::json!({"is_primary": false}))
            .await
            .unwrap();

        assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    }

    #[actix_rt::test]
    async fn delete_image_success() {
        let ctx = TestContext::new(None).await;
        let token = register_and_login(&ctx, "imgowner11", "test123").await;

        let pet_uuid = create_pet(&ctx, &token, "Buddy", "dog").await;

        let image_bytes = make_test_image(800, 600, [255, 0, 0]);
        let (_, body) = send_upload_request(
            &ctx.client,
            &ctx.addr,
            &token,
            &pet_uuid,
            image_bytes,
            "image/png",
        )
        .await;
        let image_uuid = body["uuid"].as_str().unwrap().to_string();

        // Delete the image
        let resp = ctx
            .test_server
            .delete(format!(
                "/api/v1/user/pets/{}/images/{}",
                pet_uuid, image_uuid
            ))
            .with_token(&token)
            .send()
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);

        // Verify it's gone
        let mut resp = ctx
            .test_server
            .get(format!("/api/v1/user/pets/{}/images", pet_uuid))
            .with_token(&token)
            .send()
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let images: Vec<serde_json::Value> = resp.json().await.unwrap();
        assert!(images.is_empty());

        // Second delete returns 404
        let resp = ctx
            .test_server
            .delete(format!(
                "/api/v1/user/pets/{}/images/{}",
                pet_uuid, image_uuid
            ))
            .with_token(&token)
            .send()
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::NOT_FOUND);
    }

    #[actix_rt::test]
    async fn delete_image_ownership_enforcement() {
        let ctx = TestContext::new(None).await;
        let token_a = register_and_login(&ctx, "owner_a_img2", "test123").await;
        let token_b = register_and_login(&ctx, "owner_b_img2", "test123").await;

        let pet_uuid = create_pet(&ctx, &token_a, "Buddy", "dog").await;

        let image_bytes = make_test_image(800, 600, [255, 0, 0]);
        let (_, body) = send_upload_request(
            &ctx.client,
            &ctx.addr,
            &token_a,
            &pet_uuid,
            image_bytes,
            "image/png",
        )
        .await;
        let image_uuid = body["uuid"].as_str().unwrap().to_string();

        // Owner B tries to delete Owner A's image
        let resp = ctx
            .test_server
            .delete(format!(
                "/api/v1/user/pets/{}/images/{}",
                pet_uuid, image_uuid
            ))
            .with_token(&token_b)
            .send()
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::NOT_FOUND);
    }

    #[actix_rt::test]
    async fn get_public_image_metadata() {
        let ctx = TestContext::new(None).await;
        let token = register_and_login(&ctx, "imgowner12", "test123").await;

        let pet_uuid = create_pet(&ctx, &token, "Buddy", "dog").await;

        let image_bytes = make_test_image(800, 600, [255, 0, 0]);
        let (_, body) = send_upload_request(
            &ctx.client,
            &ctx.addr,
            &token,
            &pet_uuid,
            image_bytes,
            "image/png",
        )
        .await;
        let image_uuid = body["uuid"].as_str().unwrap().to_string();

        let mut resp = ctx
            .test_server
            .get(format!("/api/v1/public/pets/images/{}", image_uuid))
            .send()
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let metadata: serde_json::Value = resp.json().await.unwrap();
        assert_eq!(metadata["uuid"], image_uuid);
        assert_eq!(metadata["format"], "webp");
    }

    #[actix_rt::test]
    async fn get_public_image_variant_stream() {
        let ctx = TestContext::new(None).await;
        let token = register_and_login(&ctx, "imgowner13", "test123").await;

        let pet_uuid = create_pet(&ctx, &token, "Buddy", "dog").await;

        let image_bytes = make_test_image(800, 600, [255, 0, 0]);
        let (_, body) = send_upload_request(
            &ctx.client,
            &ctx.addr,
            &token,
            &pet_uuid,
            image_bytes,
            "image/png",
        )
        .await;
        let image_uuid = body["uuid"].as_str().unwrap().to_string();

        let mut resp = ctx
            .test_server
            .get(format!(
                "/api/v1/public/pets/images/{}/thumbnail",
                image_uuid
            ))
            .send()
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        assert_eq!(
            resp.headers()
                .get(actix_web::http::header::CONTENT_TYPE)
                .unwrap(),
            "image/webp"
        );
        let bytes = resp.body().await.unwrap().len();
        assert!(bytes > 0);
    }

    #[actix_rt::test]
    async fn get_public_image_variant_invalid() {
        let ctx = TestContext::new(None).await;
        let token = register_and_login(&ctx, "imgowner14", "test123").await;

        let pet_uuid = create_pet(&ctx, &token, "Buddy", "dog").await;

        let image_bytes = make_test_image(800, 600, [255, 0, 0]);
        let (_, body) = send_upload_request(
            &ctx.client,
            &ctx.addr,
            &token,
            &pet_uuid,
            image_bytes,
            "image/png",
        )
        .await;
        let image_uuid = body["uuid"].as_str().unwrap().to_string();

        let resp = ctx
            .test_server
            .get(format!(
                "/api/v1/public/pets/images/{}/invalid_variant",
                image_uuid
            ))
            .send()
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    }
}
