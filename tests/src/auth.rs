use std::time::{SystemTime, UNIX_EPOCH};

use axum::body::Body;
use axum::http::{Request, Response, StatusCode};
use serde_json::json;
use sub60_cache::api::endpoints::auth_github_actions;
use sub60_cache::api::{ApiPath, Endpoint, PathParams};
use sub60_cache::{Context, HostSoftware, TokenFormat, TokenKind};
use tower_service::Service;

use crate::{CiJobGrant, MockOidcVerifier, ORIGIN, TestContext};

#[tokio::test]
async fn github_actions_token_is_exchanged_if_the_job_can_access_a_cache() {
    let test_ctx = test_ctx_with_grant();

    let response = exchange(&test_ctx, &sign(&claims())).await;
    assert_eq!(response.status(), StatusCode::OK);

    let body = parse_body(response).await;

    let token = test_ctx.token_format().parse(&body.token).unwrap();
    assert_eq!(token.kind(), TokenKind::CiJob);

    let stored = test_ctx.db().ci_job_token(&token.hash()).unwrap();
    assert_eq!(stored.expires_at, body.expires_at);

    let ttl = body.expires_at.as_secs() - now();
    assert!((3590..=3600).contains(&ttl), "{ttl}");

    let job = stored.job;
    assert_eq!(job.host, "https://github.com");
    assert_eq!(job.host_software, HostSoftware::GitHub);
    assert_eq!(job.host_namespace_id, "583231");
    assert_eq!(job.host_repo_id, "1296269");
    assert_eq!(job.host_repo_name, "Hello-World");
    assert_eq!(job.git_ref, "refs/heads/main");
    assert_eq!(
        job.commit_sha.to_string(),
        "7fd1a60b01f91b314f59955a4e4d4e80d8edf11d"
    );
    assert_eq!(job.host_run_id, "41171495766");
    assert_eq!(job.run_attempt, Some(2));
    assert_eq!(
        job.run_url,
        "https://github.com/octocat/Hello-World/actions/runs/41171495766/\
         attempts/2"
    );
}

#[tokio::test]
async fn github_actions_token_is_not_exchanged_if_the_job_cannot_access_any_cache()
 {
    let mut test_ctx = TestContext::default();
    test_ctx.db_mut().add_namespace("octocat");
    test_ctx.db_mut().add_cache("octocat", "Hello-World");
    test_ctx.db_mut().add_ci_job_grant(
        "octocat",
        "Hello-World",
        CiJobGrant { git_ref: Some("refs/heads/release".into()), ..grant() },
    );

    let response = exchange(&test_ctx, &sign(&claims())).await;
    assert_eq!(response.status(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn github_actions_token_must_be_for_this_server() {
    let mut claims = claims();
    claims["aud"] = json!("https://other.example.com");
    assert_unauthorized(&sign(&claims)).await;

    claims["aud"] = json!(["https://other.example.com", ORIGIN]);
    let response = exchange(&test_ctx_with_grant(), &sign(&claims)).await;
    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn github_actions_token_must_be_issued_by_github_actions() {
    let mut claims = claims();
    claims["iss"] = json!("https://example.com");
    assert_unauthorized(&sign(&claims)).await;
}

#[tokio::test]
async fn github_actions_token_must_not_be_expired() {
    let mut claims = claims();
    claims["exp"] = json!(now() - 120);
    assert_unauthorized(&sign(&claims)).await;
}

#[tokio::test]
async fn github_actions_token_must_already_be_valid() {
    let mut claims = claims();
    claims["nbf"] = json!(now() + 120);
    assert_unauthorized(&sign(&claims)).await;
}

#[tokio::test]
async fn github_actions_token_must_have_a_valid_signature() {
    let jwt = MockOidcVerifier::sign(
        &json!({ "alg": "RS256", "kid": MockOidcVerifier::KEY_ID }),
        &claims(),
        "other-key",
    );
    assert_unauthorized(&jwt).await;

    // A valid signature, but of different claims.
    let mut other_claims = claims();
    other_claims["repository_owner_id"] = json!("1");
    let other_jwt = sign(&other_claims);
    let jwt = sign(&claims());
    let (message, _) = jwt.rsplit_once('.').unwrap();
    let (_, other_signature) = other_jwt.rsplit_once('.').unwrap();
    assert_unauthorized(&format!("{message}.{other_signature}")).await;
}

#[tokio::test]
async fn github_actions_token_must_be_signed_with_rs256() {
    for alg in ["none", "HS256", "rs256"] {
        let jwt = MockOidcVerifier::sign(
            &json!({ "alg": alg, "kid": MockOidcVerifier::KEY_ID }),
            &claims(),
            MockOidcVerifier::KEY_ID,
        );
        assert_unauthorized(&jwt).await;
    }
}

#[tokio::test]
async fn github_actions_token_must_name_its_key() {
    let jwt = MockOidcVerifier::sign(
        &json!({ "alg": "RS256" }),
        &claims(),
        MockOidcVerifier::KEY_ID,
    );
    assert_unauthorized(&jwt).await;
}

#[tokio::test]
async fn github_actions_token_must_have_a_numeric_run_attempt() {
    let mut claims = claims();
    claims["run_attempt"] = json!("first");
    assert_unauthorized(&sign(&claims)).await;
}

#[tokio::test]
async fn github_actions_token_must_be_a_jwt() {
    for jwt in ["", "foo", "a.b", "a.b.c", "a.b.c.d"] {
        assert_unauthorized(jwt).await;
    }
}

/// Returns the claims of a token GitHub could have signed for a job of
/// `octocat/Hello-World`.
fn claims() -> serde_json::Value {
    let now = now();

    json!({
        "iss": "https://token.actions.githubusercontent.com",
        "aud": ORIGIN,
        "iat": now,
        "nbf": now,
        "exp": now + 300,
        "repository": "octocat/Hello-World",
        "repository_id": "1296269",
        "repository_owner": "octocat",
        "repository_owner_id": "583231",
        "ref": "refs/heads/main",
        "sha": "7fd1a60b01f91b314f59955a4e4d4e80d8edf11d",
        "run_id": "41171495766",
        "run_attempt": "2",
    })
}

/// Returns a grant that gives access to the jobs of `octocat/Hello-World`.
fn grant() -> CiJobGrant {
    CiJobGrant {
        host: "https://github.com".into(),
        host_namespace_id: "583231".into(),
        host_repo_name: Some("hello-world".into()),
        git_ref: None,
    }
}

fn test_ctx_with_grant() -> TestContext {
    let mut test_ctx = TestContext::default();
    test_ctx.db_mut().add_namespace("octocat");
    test_ctx.db_mut().add_cache("octocat", "Hello-World");
    test_ctx.db_mut().add_ci_job_grant("octocat", "Hello-World", grant());
    test_ctx
}

fn sign(claims: &serde_json::Value) -> String {
    MockOidcVerifier::sign(
        &json!({ "alg": "RS256", "kid": MockOidcVerifier::KEY_ID }),
        claims,
        MockOidcVerifier::KEY_ID,
    )
}

async fn assert_unauthorized(jwt: &str) {
    let test_ctx = test_ctx_with_grant();
    let response = exchange(&test_ctx, jwt).await;
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED, "{jwt}");
    assert!(test_ctx.db().num_ci_job_tokens() == 0);
}

async fn exchange(test_ctx: &TestContext, jwt: &str) -> Response<Body> {
    let body = auth_github_actions::RequestBody { token: jwt.to_owned() };

    let request = Request::builder()
        .method(auth_github_actions::Endpoint::METHOD)
        .uri(auth_github_actions::Path { api: ApiPath }.to_path())
        .body(Body::from(body.to_string()))
        .expect("valid request");

    let Ok(response) = test_ctx.app_router().call(request).await;

    response
}

async fn parse_body(
    response: Response<Body>,
) -> auth_github_actions::ResponseBody {
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("read response body");

    str::from_utf8(&body)
        .expect("response body is UTF-8")
        .parse()
        .expect("valid response body")
}

fn now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("we're past the UNIX epoch")
        .as_secs()
}
