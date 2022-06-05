pub(crate) mod guests;
pub(crate) mod vote;
pub(crate) mod vtubers;

use axum::{
    extract::rejection::ExtensionRejection,
    response::{IntoResponse, Response},
};
use deadpool_postgres::PoolError;
use hyper::StatusCode;

/// All (or most) of the possible errors that can happen in Oshi Smash.
pub enum Error {
    UnableToQuery(tokio_postgres::Error),
    FailedToSetupDb(deadpool_postgres::BuildError),
    InvalidGuest,
    PoolError(PoolError),
    FailedToParseVoteEntry,
    MissingDbHandleExtension,
    MaxVisitedIsLessThanCurrent,

    // TODO: Choose one only
    FailedToParseStack(vtubers::Error),
    StackParseFailed,
    InvalidClientData,
    NotAllowedToVote,
}

impl From<tokio_postgres::Error> for Error {
    fn from(e: tokio_postgres::Error) -> Self {
        Error::UnableToQuery(e)
    }
}

impl From<deadpool_postgres::BuildError> for Error {
    fn from(e: deadpool_postgres::BuildError) -> Self {
        Error::FailedToSetupDb(e)
    }
}

impl From<PoolError> for Error {
    fn from(e: PoolError) -> Self {
        Error::PoolError(e)
    }
}

impl From<vote::Error> for Error {
    fn from(_: vote::Error) -> Self {
        Error::FailedToParseVoteEntry
    }
}

impl From<vtubers::Error> for Error {
    fn from(e: vtubers::Error) -> Self {
        Error::FailedToParseStack(e)
    }
}

impl From<serde_json::Error> for Error {
    fn from(_: serde_json::Error) -> Self {
        Error::StackParseFailed
    }
}

impl From<ExtensionRejection> for Error {
    fn from(_: ExtensionRejection) -> Self {
        Error::MissingDbHandleExtension
    }
}

// TODO: Move out to `oshismash_web`
impl IntoResponse for Error {
    fn into_response(self: Error) -> Response {
        match self {
            Error::UnableToQuery(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "E001: Failed when attempting to query the database",
            ),
            Error::FailedToSetupDb(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "E002: Failed to setup database connection",
            ),
            Error::InvalidGuest => (StatusCode::UNAUTHORIZED, "E003: Not allowed to access this"),
            Error::PoolError(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "E004: Failed to get client from DB pool",
            ),
            Error::FailedToParseVoteEntry => {
                (StatusCode::BAD_REQUEST, "E005: Failed to parse vote entry")
            }
            Error::FailedToParseStack(_e) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "E006: Failed to parse card stack",
            ),
            Error::StackParseFailed => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "E006: Failed to parse card stack",
            ),
            Error::MissingDbHandleExtension => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "E007: Missing DB handle extension",
            ),
            Error::InvalidClientData => (
                StatusCode::BAD_REQUEST,
                "E008: You can only vote for a VTuber that's currently displayed",
            ),
            Error::NotAllowedToVote => (
                StatusCode::FORBIDDEN,
                "E009: You have to vote for the previous entries first.",
            ),
            Error::MaxVisitedIsLessThanCurrent => (
                StatusCode::FORBIDDEN,
                "E010: You have to vote for the previous entries first.",
            ),
        }
        .into_response()
    }
}
