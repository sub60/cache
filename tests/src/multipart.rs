use core::num::NonZeroU16;

use axum::body::Body;
use axum::http::{Method, Request, StatusCode};
use nix_types::NarFileName;
use sub60_cache::nar_multiparts::{CompleteRequestBody, Etag};
use sub60_cache::{Context, ObjectStorage, ObjectStorageKey};
use tower_service::Service;

use crate::{MockNixClient, NixClient, NixStore, TestContext};

#[tokio::test]
async fn completion_distinguishes_insert_from_race_loss() {
    let mut test_ctx = TestContext::default();
    test_ctx.db_mut().add_user("user");
    test_ctx.db_mut().add_cache("user", "cache");

    let nix = MockNixClient::default();
    let store_path = nix.store().add_file("hello-world", b"hello world").await;
    let (narinfo, nar_bytes) =
        nix.store().nar_info_and_bytes(&store_path).await;
    let nar_file_name = NarFileName::from(&narinfo);

    let object_storage = test_ctx.object_storage().clone();
    let first_upload_id = object_storage
        .create_multipart_upload(ObjectStorageKey::Nar(&nar_file_name))
        .await
        .unwrap();
    let second_upload_id = object_storage
        .create_multipart_upload(ObjectStorageKey::Nar(&nar_file_name))
        .await
        .unwrap();

    let first_etag = object_storage
        .upload_multipart_part(
            first_upload_id,
            NonZeroU16::MIN,
            nar_bytes.clone(),
        )
        .unwrap();
    let second_etag = object_storage
        .upload_multipart_part(second_upload_id, NonZeroU16::MIN, nar_bytes)
        .unwrap();

    let mut app = test_ctx.app_router();

    let first_response = complete_upload(
        &mut app,
        first_upload_id,
        nar_file_name.clone(),
        first_etag,
    )
    .await;
    assert_eq!(first_response, StatusCode::CREATED);

    let second_response =
        complete_upload(&mut app, second_upload_id, nar_file_name, second_etag)
            .await;
    assert_eq!(second_response, StatusCode::OK);
}

#[tokio::test]
async fn completion_of_unknown_upload_is_not_found() {
    let mut test_ctx = TestContext::default();
    test_ctx.db_mut().add_user("user");
    test_ctx.db_mut().add_cache("user", "cache");

    let nix = MockNixClient::default();
    let store_path = nix.store().add_file("hello-world", b"hello world").await;
    let (narinfo, _) = nix.store().nar_info_and_bytes(&store_path).await;
    let nar_file_name = NarFileName::from(&narinfo);

    let mut app = test_ctx.app_router();

    let response =
        complete_upload(&mut app, 42, nar_file_name, placeholder_etag()).await;
    assert_eq!(response, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn completion_with_invalid_parts_is_bad_request() {
    let mut test_ctx = TestContext::default();
    test_ctx.db_mut().add_user("user");
    test_ctx.db_mut().add_cache("user", "cache");

    let nix = MockNixClient::default();
    let store_path = nix.store().add_file("hello-world", b"hello world").await;
    let (narinfo, _) = nix.store().nar_info_and_bytes(&store_path).await;
    let nar_file_name = NarFileName::from(&narinfo);

    // Create an upload without uploading any parts to it.
    let upload_id = test_ctx
        .object_storage()
        .create_multipart_upload(ObjectStorageKey::Nar(&nar_file_name))
        .await
        .unwrap();

    let mut app = test_ctx.app_router();

    let response =
        complete_upload(&mut app, upload_id, nar_file_name, placeholder_etag())
            .await;
    assert_eq!(response, StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn initiation_skips_existing_nars() {
    let mut test_ctx = TestContext::default();
    test_ctx.db_mut().add_user("user");
    test_ctx.db_mut().add_cache("user", "cache");

    let nix = MockNixClient::default();
    let store_path = nix.store().add_file("hello-world", b"hello world").await;
    let (narinfo, nar_bytes) =
        nix.store().nar_info_and_bytes(&store_path).await;
    let nar_file_name = NarFileName::from(&narinfo);

    let mut app = test_ctx.app_router();

    let response = initiate_upload(&mut app, &nar_file_name).await;
    assert_eq!(response, StatusCode::OK);

    test_ctx
        .object_storage()
        .insert(ObjectStorageKey::Nar(&nar_file_name), nar_bytes)
        .await
        .unwrap();

    let response = initiate_upload(&mut app, &nar_file_name).await;
    assert_eq!(response, StatusCode::NO_CONTENT);
}

async fn initiate_upload(
    app: &mut axum::Router,
    nar_file_name: &NarFileName,
) -> StatusCode {
    let request = Request::builder()
        .method(Method::POST)
        .uri("/user/cache/nar-multiparts")
        .body(Body::from(nar_file_name.to_string()))
        .expect("valid request");

    match app.call(request).await {
        Ok(response) => response.status(),
    }
}

fn placeholder_etag() -> Etag {
    "00000000000000000000000000000000".parse().expect("valid Etag")
}

async fn complete_upload(
    app: &mut axum::Router,
    upload_id: u64,
    nar_file_name: NarFileName,
    etag: Etag,
) -> StatusCode {
    let body = CompleteRequestBody {
        etags: core::iter::once(etag).collect(),
        nar_file_name,
    };

    let request = Request::builder()
        .method(Method::POST)
        .uri(format!("/user/cache/nar-multiparts/{upload_id}/complete"))
        .body(Body::from(body.to_string()))
        .expect("valid request");

    match app.call(request).await {
        Ok(response) => response.status(),
    }
}
