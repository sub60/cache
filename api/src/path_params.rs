use core::fmt;
use core::str::FromStr;

#[cfg(feature = "axum")]
pub use extract::PathParamsRejection;
use percent_encoding::{AsciiSet, NON_ALPHANUMERIC, utf8_percent_encode};
pub use sub60_cache_api_macros::PathParams;

/// The parameters in an endpoint's path, along with the path's template.
///
/// It's usually derived, with the path given by a `#[route(..)]` attribute.
pub trait PathParams: Sized {
    /// The template of the path, in which each `{name}` stands for the
    /// parameter with that name.
    const PATH: &'static str;

    /// Appends the path with the parameters filled in, each percent-encoded.
    fn write_path(&self, path: &mut String);

    /// Parses the parameters from the ones matched in a request's path, as
    /// (name, value) pairs in the order they appear in the path, with their
    /// values percent-decoded.
    ///
    /// It only takes the pairs it needs, so the ones after them are left for
    /// whatever comes next in the path.
    ///
    /// # Errors
    ///
    /// Returns an error if a pair is missing or isn't the one expected, or if
    /// a value doesn't parse.
    fn parse<'a>(
        params: &mut impl Iterator<Item = (&'a str, &'a str)>,
    ) -> Result<Self, PathParamsError>;

    /// Returns the path with the parameters filled in, each percent-encoded.
    #[inline]
    fn to_path(&self) -> String {
        let mut path = String::new();
        self.write_path(&mut path);
        path
    }
}

/// The type of error that can occur when parsing [`PathParams`].
#[derive(Debug, Clone, PartialEq, Eq, derive_more::Display, cauchy::Error)]
pub enum PathParamsError {
    /// There was no parameter where one was expected.
    #[display("missing path parameter {_0:?}")]
    Missing(&'static str),

    /// The parameter where one was expected had a different name.
    #[display("expected path parameter {expected:?}, found {found:?}")]
    UnexpectedName {
        /// The name of the expected parameter.
        expected: &'static str,
        /// The name of the parameter that was found instead.
        found: String,
    },

    /// A parameter's value didn't parse.
    #[display("invalid path parameter {name:?}: {message}")]
    Invalid {
        /// The parameter's name.
        name: &'static str,
        /// Why the value didn't parse.
        message: String,
    },
}

/// Items the derive macro expands to.
#[doc(hidden)]
pub mod __private {
    #[cfg(feature = "axum")]
    pub use ::axum;
    pub use const_format::concatcp;

    #[cfg(feature = "axum")]
    pub use super::extract::from_request_parts;
    use super::*;

    /// The characters that get percent-encoded in a path parameter, i.e. all
    /// of them but the unreserved ones.
    const PATH_PARAM: &AsciiSet =
        &NON_ALPHANUMERIC.remove(b'-').remove(b'.').remove(b'_').remove(b'~');

    #[inline]
    pub fn push_param(path: &mut String, value: &impl fmt::Display) {
        path.extend(utf8_percent_encode(&value.to_string(), PATH_PARAM));
    }

    #[inline]
    pub fn parse_param<'a, T>(
        params: &mut impl Iterator<Item = (&'a str, &'a str)>,
        name: &'static str,
    ) -> Result<T, PathParamsError>
    where
        T: FromStr<Err: fmt::Display>,
    {
        let (found, value) =
            params.next().ok_or(PathParamsError::Missing(name))?;

        if found != name {
            return Err(PathParamsError::UnexpectedName {
                expected: name,
                found: found.to_owned(),
            });
        }

        value.parse().map_err(|err: T::Err| PathParamsError::Invalid {
            name,
            message: err.to_string(),
        })
    }
}

/// Extracting [`PathParams`] from axum requests.
#[cfg(feature = "axum")]
mod extract {
    use axum::extract::rejection::RawPathParamsRejection;
    use axum::extract::{FromRequestParts, RawPathParams};
    use axum::http::StatusCode;
    use axum::http::request::Parts;
    use axum::response::{IntoResponse, Response};

    use crate::{PathParams, PathParamsError};

    /// The rejection returned when [`PathParams`] can't be extracted from a
    /// request.
    #[derive(Debug, derive_more::Display, cauchy::Error)]
    pub enum PathParamsRejection {
        /// The path's parameters couldn't be read.
        #[display("{_0}")]
        Raw(RawPathParamsRejection),

        /// The path's parameters didn't parse.
        #[display("{_0}")]
        Parse(PathParamsError),
    }

    /// Extracts the path parameters of a request, for the `FromRequestParts` impls
    /// the derive macro expands to.
    #[doc(hidden)]
    pub async fn from_request_parts<T: PathParams, S: Send + Sync>(
        parts: &mut Parts,
        state: &S,
    ) -> Result<T, PathParamsRejection> {
        let params = RawPathParams::from_request_parts(parts, state)
            .await
            .map_err(PathParamsRejection::Raw)?;

        T::parse(&mut params.iter()).map_err(PathParamsRejection::Parse)
    }

    impl IntoResponse for PathParamsRejection {
        fn into_response(self) -> Response {
            match self {
                Self::Raw(rejection) => rejection.into_response(),
                Self::Parse(err @ PathParamsError::Invalid { .. }) => {
                    (StatusCode::BAD_REQUEST, err.to_string()).into_response()
                },
                // The route's template and the parameters disagree, which is a
                // bug on the server.
                Self::Parse(
                    err @ (PathParamsError::Missing(_)
                    | PathParamsError::UnexpectedName { .. }),
                ) => (StatusCode::INTERNAL_SERVER_ERROR, err.to_string())
                    .into_response(),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{PathParams, PathParamsError};

    #[derive(Debug, PartialEq, Eq, PathParams)]
    #[route("/{owner}/{name}")]
    struct RepoPath {
        owner: String,
        name: String,
    }

    #[derive(Debug, PartialEq, Eq, PathParams)]
    #[route("/lines/{line}")]
    struct LinesPath {
        line: u32,
    }

    #[derive(Debug, PartialEq, Eq, PathParams)]
    #[route(RepoPath + "/files/{file_id}" + LinesPath)]
    struct FilePath<Id = String> {
        repo: RepoPath,
        file_id: Id,
        lines: LinesPath,
    }

    #[derive(Debug, PartialEq, Eq, PathParams)]
    #[route("/-/static")]
    struct StaticPath;

    fn repo() -> RepoPath {
        RepoPath { owner: "octocat".to_owned(), name: "Hello-World".to_owned() }
    }

    fn file(file_id: &str) -> FilePath {
        FilePath {
            repo: repo(),
            file_id: file_id.to_owned(),
            lines: LinesPath { line: 3 },
        }
    }

    #[test]
    fn path_joins_the_segments() {
        assert_eq!(
            FilePath::<String>::PATH,
            "/{owner}/{name}/files/{file_id}/lines/{line}"
        );
        assert_eq!(StaticPath::PATH, "/-/static");
    }

    #[test]
    fn to_path_fills_in_the_parameters() {
        assert_eq!(
            file("abc").to_path(),
            "/octocat/Hello-World/files/abc/lines/3"
        );
        assert_eq!(StaticPath.to_path(), "/-/static");
    }

    #[test]
    fn to_path_percent_encodes_all_but_unreserved_characters() {
        assert_eq!(
            file("a/b?c d%-._~").to_path(),
            "/octocat/Hello-World/files/a%2Fb%3Fc%20d%25-._~/lines/3"
        );
    }

    #[test]
    fn parse_reads_the_parameters_in_order() {
        let params = [
            ("owner", "octocat"),
            ("name", "Hello-World"),
            ("file_id", "a/b"),
            ("line", "3"),
        ];

        assert_eq!(
            FilePath::<String>::parse(&mut params.into_iter()),
            Ok(file("a/b"))
        );
    }

    #[test]
    fn parse_leaves_the_parameters_it_doesnt_need() {
        let mut params =
            [("owner", "octocat"), ("name", "Hello-World"), ("file_id", "abc")]
                .into_iter();

        assert_eq!(RepoPath::parse(&mut params), Ok(repo()));
        assert_eq!(params.next(), Some(("file_id", "abc")));
    }

    #[test]
    fn parse_rejects_missing_parameters() {
        let params = [("owner", "octocat")];

        assert_eq!(
            RepoPath::parse(&mut params.into_iter()),
            Err(PathParamsError::Missing("name"))
        );
    }

    #[test]
    fn parse_rejects_unexpected_parameters() {
        let params = [("owner", "octocat"), ("file_id", "abc")];

        assert_eq!(
            RepoPath::parse(&mut params.into_iter()),
            Err(PathParamsError::UnexpectedName {
                expected: "name",
                found: "file_id".to_owned(),
            })
        );
    }

    #[test]
    fn parse_rejects_invalid_values() {
        let params = [("line", "third")];

        assert!(matches!(
            LinesPath::parse(&mut params.into_iter()),
            Err(PathParamsError::Invalid { name: "line", .. })
        ));
    }
}
