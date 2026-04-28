//! Extractors for request bodies parsed from strings.

use core::fmt;
use core::str::FromStr;

use axum::extract::rejection::StringRejection;
use axum::extract::{FromRequest, Request};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};

/// Extracts a request body as UTF-8 text and parses it with [`FromStr`].
pub(crate) struct FromStrBody<T>(pub(crate) T);

/// The rejection returned by [`FromStrBody`].
#[derive(Debug, derive_more::Display)]
pub(crate) enum FromStrBodyRejection<ParseError> {
    /// The request body could not be extracted as UTF-8 text.
    #[display("request body is not UTF-8: {_0}")]
    String(StringRejection),

    /// The request body did not parse successfully.
    #[display("invalid request body: {_0}")]
    Parse(ParseError),
}

impl<T, S> FromRequest<S> for FromStrBody<T>
where
    S: Send + Sync,
    T: FromStr,
    T::Err: fmt::Display,
{
    type Rejection = FromStrBodyRejection<T::Err>;

    async fn from_request(
        req: Request,
        state: &S,
    ) -> Result<Self, Self::Rejection> {
        String::from_request(req, state)
            .await
            .map_err(FromStrBodyRejection::String)?
            .parse()
            .map(Self)
            .map_err(FromStrBodyRejection::Parse)
    }
}

impl<ParseError> IntoResponse for FromStrBodyRejection<ParseError>
where
    ParseError: fmt::Display,
{
    fn into_response(self) -> Response {
        match self {
            Self::String(err) => err.into_response(),
            Self::Parse(err) => {
                (StatusCode::BAD_REQUEST, err.to_string()).into_response()
            },
        }
    }
}
