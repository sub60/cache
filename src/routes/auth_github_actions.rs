//! TODO: docs.

use core::time::Duration;

use axum::http::{StatusCode, header};
use axum::response::IntoResponse;
use axum::routing::{self, MethodRouter};
use smol_str::SmolStr;

use crate::auth::{AuthResponseBody, GitHubActionsAuthRequestBody};
use crate::from_str_body::FromStrBody;
use crate::jwt::{self, JwtError};
use crate::token::{Token, TokenFormat as _, TokenKind};
use crate::{
    CacheDb,
    CiJob,
    Clock,
    CommitSha,
    Context,
    HostSoftware,
    OidcIssuer,
    OidcVerifier,
};

pub(crate) const ROUTE: &str = "/auth/github-actions";

/// How long the tokens given to jobs are valid for.
const TOKEN_TTL: Duration = Duration::from_hours(1);

/// The host GitHub Actions jobs on `github.com` run on.
const HOST: &str = "https://github.com";

type Error<Ctx> = AuthGitHubActionsError<
    <<Ctx as Context>::OidcVerifier as OidcVerifier>::Error,
    <<Ctx as Context>::CacheDb as CacheDb>::Error,
>;

/// The claims GitHub signs in the OIDC tokens of GitHub Actions jobs, on top of
/// the registered ones.
#[derive(serde::Deserialize)]
struct GitHubActionsClaims {
    /// The repository's full name, e.g. `octocat/hello-world`.
    repository: String,
    repository_id: SmolStr,
    /// The name of the repository's owner, e.g. `octocat`.
    repository_owner: SmolStr,
    repository_owner_id: SmolStr,
    #[serde(rename = "ref")]
    git_ref: SmolStr,
    sha: CommitSha,
    run_id: SmolStr,
    #[serde(deserialize_with = "crate::serde_utils::deserialize_from_str")]
    run_attempt: u32,
}

#[derive(Debug, derive_more::Display, cauchy::Error)]
enum AuthGitHubActionsError<OidcVerifierError, CacheDbError> {
    #[display("{_0}")]
    Jwt(JwtError<OidcVerifierError>),

    #[display("the token's {_0:?} claim is invalid")]
    InvalidClaim(&'static str),

    #[display("the job can't access any cache")]
    NoAccess,

    #[display("couldn't store the token: {_0}")]
    InsertToken(CacheDbError),
}

pub(crate) fn router(mut ctx: impl Context) -> MethodRouter {
    routing::post(async move |body| handler(body, &mut ctx).await)
}

async fn handler<Ctx: Context>(
    FromStrBody(body): FromStrBody<GitHubActionsAuthRequestBody>,
    ctx: &mut Ctx,
) -> Result<impl IntoResponse + use<Ctx>, Error<Ctx>> {
    let claims = jwt::verify::<GitHubActionsClaims, _>(
        &body.token,
        OidcIssuer::GitHubActions,
        ctx,
    )
    .await
    .map_err(AuthGitHubActionsError::Jwt)?;

    let job = claims.into_ci_job()?;

    let now = ctx.clock().unix_ts();

    let token = Token::generate(TokenKind::CiJob, &mut ctx.rng());

    let expires_at = Duration::from_secs((now + TOKEN_TTL).as_secs());

    let was_inserted = ctx
        .cache_db()
        .insert_ci_job_token(token.hash(), &job, expires_at)
        .await
        .map_err(AuthGitHubActionsError::InsertToken)?;

    if !was_inserted {
        return Err(AuthGitHubActionsError::NoAccess);
    }

    let body = AuthResponseBody {
        token: ctx.token_format().display(&token).to_string(),
        expires_at,
    };

    Ok(([(header::CONTENT_TYPE, "application/json")], body.to_string()))
}

impl GitHubActionsClaims {
    fn into_ci_job<OidcVerifierError, CacheDbError>(
        self,
    ) -> Result<CiJob, AuthGitHubActionsError<OidcVerifierError, CacheDbError>>
    {
        let host_repo_name = self
            .repository
            .strip_prefix(&*self.repository_owner)
            .and_then(|rest| rest.strip_prefix('/'))
            .ok_or(AuthGitHubActionsError::InvalidClaim("repository"))?
            .into();

        let run_url = format!(
            "{HOST}/{}/actions/runs/{}/attempts/{}",
            self.repository, self.run_id, self.run_attempt,
        );

        Ok(CiJob {
            host: HOST.into(),
            host_software: HostSoftware::GitHub,
            host_namespace_id: self.repository_owner_id,
            host_repo_id: self.repository_id,
            host_repo_name,
            git_ref: self.git_ref,
            commit_sha: self.sha,
            host_run_id: self.run_id,
            run_attempt: Some(self.run_attempt),
            run_url,
        })
    }
}

impl<OidcVerifierError, CacheDbError>
    AuthGitHubActionsError<OidcVerifierError, CacheDbError>
where
    OidcVerifierError: Into<StatusCode>,
    CacheDbError: Into<StatusCode>,
{
    fn into_status_code(self) -> StatusCode {
        match self {
            Self::Jwt(JwtError::Verifier(err)) => err.into(),
            Self::InsertToken(err) => err.into(),
            Self::Jwt(_) | Self::InvalidClaim(_) => StatusCode::UNAUTHORIZED,
            Self::NoAccess => StatusCode::FORBIDDEN,
        }
    }
}

impl<OidcVerifierError, CacheDbError> IntoResponse
    for AuthGitHubActionsError<OidcVerifierError, CacheDbError>
where
    Self: core::fmt::Display,
    OidcVerifierError: Into<StatusCode>,
    CacheDbError: Into<StatusCode>,
{
    fn into_response(self) -> axum::response::Response {
        let message = self.to_string();
        (self.into_status_code(), message).into_response()
    }
}
