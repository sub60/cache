use core::num::{NonZeroU16, NonZeroU32};

use axum::body::Body;
use axum::http::{Method, Request, StatusCode, header};
use nix_types::{NarFileName, NarInfo};
use sub60_cache::api::Etag;
use sub60_cache::api::endpoints::{
    nar_multiparts,
    nar_multiparts_complete,
    nar_multiparts_parts,
};
use sub60_cache::{Context, NarStorage};
use tower_service::Service;

use crate::{MockNixClient, NixClient, NixStore, TestContext};

#[tokio::test]
async fn completion_distinguishes_insert_from_race_loss() {
    let mut test_ctx = TestContext::default();
    test_ctx.db_mut().add_namespace("namespace");
    let cache = test_ctx.db_mut().add_cache("namespace", "cache");
    let token = test_ctx.write_token("namespace", "cache");

    let nix = MockNixClient::default();
    let store_path = nix.store().add_file("hello-world", b"hello world").await;
    let (narinfo, nar_bytes) =
        nix.store().nar_info_and_bytes(&store_path).await;
    let nar_file_name = NarFileName::from(&narinfo);

    let nar_storage = test_ctx.nar_storage().clone();
    let first_upload_id = nar_storage
        .create_multipart_upload(&cache, &nar_file_name, narinfo.file_size)
        .await
        .unwrap();
    let second_upload_id = nar_storage
        .create_multipart_upload(&cache, &nar_file_name, narinfo.file_size)
        .await
        .unwrap();

    let first_etag = nar_storage
        .upload_multipart_part(
            first_upload_id,
            NonZeroU16::MIN,
            nar_bytes.clone(),
        )
        .unwrap();
    let second_etag = nar_storage
        .upload_multipart_part(second_upload_id, NonZeroU16::MIN, nar_bytes)
        .unwrap();

    let mut app = test_ctx.app_router();

    let first_response = complete_upload(
        &mut app,
        &token,
        first_upload_id,
        nar_file_name.clone(),
        first_etag,
    )
    .await;
    assert_eq!(first_response, StatusCode::CREATED);

    let second_response = complete_upload(
        &mut app,
        &token,
        second_upload_id,
        nar_file_name,
        second_etag,
    )
    .await;
    assert_eq!(second_response, StatusCode::OK);
}

#[tokio::test]
async fn completion_of_unknown_upload_is_not_found() {
    let mut test_ctx = TestContext::default();
    test_ctx.db_mut().add_namespace("namespace");
    test_ctx.db_mut().add_cache("namespace", "cache");
    let token = test_ctx.write_token("namespace", "cache");

    let nix = MockNixClient::default();
    let store_path = nix.store().add_file("hello-world", b"hello world").await;
    let (narinfo, _) = nix.store().nar_info_and_bytes(&store_path).await;
    let nar_file_name = NarFileName::from(&narinfo);

    let mut app = test_ctx.app_router();

    let response = complete_upload(
        &mut app,
        &token,
        42,
        nar_file_name,
        placeholder_etag(),
    )
    .await;
    assert_eq!(response, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn completion_with_invalid_parts_is_bad_request() {
    let mut test_ctx = TestContext::default();
    test_ctx.db_mut().add_namespace("namespace");
    let cache = test_ctx.db_mut().add_cache("namespace", "cache");
    let token = test_ctx.write_token("namespace", "cache");

    let nix = MockNixClient::default();
    let store_path = nix.store().add_file("hello-world", b"hello world").await;
    let (narinfo, _) = nix.store().nar_info_and_bytes(&store_path).await;
    let nar_file_name = NarFileName::from(&narinfo);

    // Create an upload without uploading any parts to it.
    let upload_id = test_ctx
        .nar_storage()
        .create_multipart_upload(&cache, &nar_file_name, narinfo.file_size)
        .await
        .unwrap();

    let mut app = test_ctx.app_router();

    let response = complete_upload(
        &mut app,
        &token,
        upload_id,
        nar_file_name,
        placeholder_etag(),
    )
    .await;
    assert_eq!(response, StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn initiation_returns_one_url_per_part_up_to_four() {
    let mut test_ctx = TestContext::default();
    test_ctx.db_mut().add_namespace("namespace");
    test_ctx.db_mut().add_cache("namespace", "cache");
    let token = test_ctx.write_token("namespace", "cache");

    let nix = MockNixClient::default();
    let store_path = nix.store().add_file("hello-world", b"hello world").await;
    let (narinfo, _) = nix.store().nar_info_and_bytes(&store_path).await;
    let file_size = narinfo.file_size.get();

    let body = nar_multiparts::RequestBody {
        nar_file_name: NarFileName::from(&narinfo),
        file_size: narinfo.file_size,
    };

    // A NAR of one part, of two parts, and of more parts than the URLs the
    // initiation returns.
    for (part_size, num_urls) in
        [(file_size, 1), (file_size.div_ceil(2), 2), (file_size / 8, 4)]
    {
        let part_size =
            NonZeroU32::new(part_size as u32).expect("non-zero part size");
        test_ctx.nar_storage().set_part_size(part_size);

        let request = Request::builder()
            .method(Method::POST)
            .header(header::AUTHORIZATION, format!("Bearer {token}"))
            .uri("/namespace/cache/nar-multiparts")
            .body(Body::from(body.to_string()))
            .expect("valid request");

        let Ok(response) = test_ctx.app_router().call(request).await;
        assert_eq!(response.status(), StatusCode::OK);

        let response_body =
            axum::body::to_bytes(response.into_body(), usize::MAX)
                .await
                .expect("read nar-multiparts body");

        let response_body = str::from_utf8(&response_body)
            .expect("UTF-8 body")
            .parse::<nar_multiparts::ResponseBody>()
            .expect("valid nar-multiparts body");

        assert_eq!(response_body.parts_urls.len(), num_urls, "{part_size}");
    }
}

#[tokio::test]
async fn part_urls_past_the_end_of_the_nar_are_bad_requests() {
    let mut test_ctx = TestContext::default();
    test_ctx.db_mut().add_namespace("namespace");
    let cache = test_ctx.db_mut().add_cache("namespace", "cache");
    let token = test_ctx.write_token("namespace", "cache");

    let nix = MockNixClient::default();
    let store_path = nix.store().add_file("hello-world", b"hello world").await;
    let (narinfo, _) = nix.store().nar_info_and_bytes(&store_path).await;
    let nar_file_name = NarFileName::from(&narinfo);

    // The default part size makes the NAR a single part.
    let upload_id = test_ctx
        .nar_storage()
        .create_multipart_upload(&cache, &nar_file_name, narinfo.file_size)
        .await
        .unwrap();

    let mut app = test_ctx.app_router();

    for (part_number, status) in
        [(1, StatusCode::OK), (2, StatusCode::BAD_REQUEST)]
    {
        let body = nar_multiparts_parts::RequestBody {
            part_numbers: [NonZeroU16::new(part_number).unwrap()]
                .into_iter()
                .collect(),
            nar_file_name: nar_file_name.clone(),
        };

        let request = Request::builder()
            .method(Method::POST)
            .header(header::AUTHORIZATION, format!("Bearer {token}"))
            .uri(format!("/namespace/cache/nar-multiparts/{upload_id}/parts"))
            .body(Body::from(body.to_string()))
            .expect("valid request");

        let Ok(response) = app.call(request).await;
        assert_eq!(response.status(), status, "part {part_number}");
    }
}

#[tokio::test]
async fn parts_must_have_the_length_their_url_accepts() {
    let mut test_ctx = TestContext::default();
    test_ctx.db_mut().add_namespace("namespace");
    let cache = test_ctx.db_mut().add_cache("namespace", "cache");

    let nix = MockNixClient::default();
    let store_path = nix.store().add_file("hello-world", b"hello world").await;
    let (narinfo, nar_bytes) =
        nix.store().nar_info_and_bytes(&store_path).await;
    let nar_file_name = NarFileName::from(&narinfo);

    let nar_storage = test_ctx.nar_storage().clone();
    let upload_id = nar_storage
        .create_multipart_upload(&cache, &nar_file_name, narinfo.file_size)
        .await
        .unwrap();

    // One byte short of the single part the NAR's size leaves.
    let short = nar_bytes.slice(..nar_bytes.len() - 1);
    assert!(
        nar_storage
            .upload_multipart_part(upload_id, NonZeroU16::MIN, short)
            .is_err()
    );

    assert!(
        nar_storage
            .upload_multipart_part(upload_id, NonZeroU16::MIN, nar_bytes)
            .is_ok()
    );
}

#[tokio::test]
async fn initiation_skips_existing_nars() {
    let mut test_ctx = TestContext::default();
    test_ctx.db_mut().add_namespace("namespace");
    test_ctx.db_mut().add_cache("namespace", "cache");
    let token = test_ctx.write_token("namespace", "cache");

    let nix = MockNixClient::default();
    let store_path = nix.store().add_file("hello-world", b"hello world").await;
    let (narinfo, nar_bytes) =
        nix.store().nar_info_and_bytes(&store_path).await;
    let nar_file_name = NarFileName::from(&narinfo);

    let mut app = test_ctx.app_router();

    let response = initiate_upload(&mut app, &token, &narinfo).await;
    assert_eq!(response, StatusCode::OK);

    test_ctx.nar_storage().insert_unchecked(&nar_file_name, nar_bytes);

    let response = initiate_upload(&mut app, &token, &narinfo).await;
    assert_eq!(response, StatusCode::NO_CONTENT);
}

async fn initiate_upload(
    app: &mut axum::Router,
    token: &str,
    narinfo: &NarInfo<String>,
) -> StatusCode {
    let body = nar_multiparts::RequestBody {
        nar_file_name: NarFileName::from(narinfo),
        file_size: narinfo.file_size,
    };

    let request = Request::builder()
        .method(Method::POST)
        .header(header::AUTHORIZATION, format!("Bearer {token}"))
        .uri("/namespace/cache/nar-multiparts")
        .body(Body::from(body.to_string()))
        .expect("valid request");

    match app.call(request).await {
        Ok(response) => response.status(),
    }
}

fn placeholder_etag() -> Etag {
    "00000000000000000000000000000000".parse().expect("valid Etag")
}

#[expect(clippy::too_many_arguments)]
async fn complete_upload(
    app: &mut axum::Router,
    token: &str,
    upload_id: u64,
    nar_file_name: NarFileName,
    etag: Etag,
) -> StatusCode {
    let body = nar_multiparts_complete::RequestBody {
        etags: core::iter::once(etag).collect(),
        nar_file_name,
    };

    let request = Request::builder()
        .method(Method::POST)
        .header(header::AUTHORIZATION, format!("Bearer {token}"))
        .uri(format!("/namespace/cache/nar-multiparts/{upload_id}/complete"))
        .body(Body::from(body.to_string()))
        .expect("valid request");

    match app.call(request).await {
        Ok(response) => response.status(),
    }
}
