pub(crate) mod guests;
pub(crate) mod vote;
pub(crate) mod vtubers;

use axum::response::{IntoResponse, Response};
use deadpool_postgres::PoolError;
use hyper::StatusCode;

use crate::db;

#[derive(Debug)]
pub enum Error {
    UnableToQuery(tokio_postgres::Error),
    FailedToSetupDb(db::Error),
    InvalidGuest,
    PoolError(PoolError),
    FailedToParseVoteEntry,

    // TODO: Choose one only
    FailedToParseStack(vtubers::Error),
    StackParseFailed,
}

impl From<tokio_postgres::Error> for Error {
    fn from(e: tokio_postgres::Error) -> Self {
        Error::UnableToQuery(e)
    }
}

impl From<db::Error> for Error {
    fn from(e: db::Error) -> Self {
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

// TODO: Implement this
// impl Serialize for Error {
//     fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
//     where
//         S: serde::Serializer {
//         todo!()
//     }
// }

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
        }
        .into_response()
    }
}
