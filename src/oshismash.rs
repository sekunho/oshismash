pub(crate) mod guests;
pub(crate) mod vote;
pub(crate) mod vtubers;

use axum::{
    extract::rejection::{ExtensionRejection, FormRejection},
    response::{IntoResponse, Response},
};
use deadpool_postgres::PoolError;
use hyper::StatusCode;

/// All (or most) of the possible errors that can happen in Oshi Smash.
#[derive(thiserror::Error, Debug)]
pub enum Error {
    // TODO: Remove this cause each handler has a more specific error now
    #[error("failed to perform the DB query ({0})")]
    UnableToQuery(tokio_postgres::Error),
    #[error("failed to setup the database connection ({0})")]
    FailedToSetupDb(deadpool_postgres::BuildError),
    #[error("not a valid guest")]
    InvalidGuest,
    #[error("unable to communicate with DB pool ({0})")]
    PoolError(PoolError),

    #[error("unable to parse JSON value to a `Vote` ({0})")]
    VoteParseError(vote::ParseError),
    #[error("vote may have failed ({0})")]
    VoteError(vote::VoteError),

    #[error("couldn't find the DB handle extension")]
    MissingDbHandleExtension,
    #[error("need to vote for previous vtubers")]
    MaxVisitedIsLessThanCurrent,
    #[error("invalid form")]
    InvalidForm(FormRejection),
    #[error("not the expected form format. e.g expected an object but got a string")]
    InvalidFormFormat,
    #[error("couldn't find the extension")]
    MissingExtension,

    // TODO: Choose one only
    #[error("couldn't parse into a `Stack` ({0})")]
    FailedToParseStack(vtubers::Error),
    #[error("couldn't parse JSON value into `Stack`")]
    StackParseFailed,
    #[error("invalid client-sourced data (like data from cookies)")]
    InvalidClientData,
    #[error("ya banned from voting")]
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

impl From<vote::ParseError> for Error {
    fn from(e: vote::ParseError) -> Self {
        Error::VoteParseError(e)
    }
}

impl From<vote::VoteError> for Error {
    fn from(e: vote::VoteError) -> Self {
        Error::VoteError(e)
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
            Error::VoteParseError(e) => {
                println!("{}", e);
                (
                    StatusCode::UNPROCESSABLE_ENTITY,
                    "E005: Failed to parse vote entry",
                )
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
                StatusCode::UNPROCESSABLE_ENTITY,
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
            Error::InvalidForm(_) => (StatusCode::BAD_REQUEST, "E11: Form data is not valid"),
            Error::MissingExtension => todo!(),
            Error::InvalidFormFormat => (
                StatusCode::UNPROCESSABLE_ENTITY,
                "E12: Form data is not of the expected format.",
            ),
            Error::VoteError(e) => match e {
                vote::VoteError::QueryPrepFailed(_) => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "E13: Vote was not counted. Something went wrong in the server.",
                ),
                vote::VoteError::QueryFailed(_) => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "E13: Vote was not counted. Something went wrong in the server.",
                ),
                vote::VoteError::InvalidDbValue => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "E14: Vote was counted but something went wrong while handling the DB result.",
                ),
            },
        }
        .into_response()
    }
}
