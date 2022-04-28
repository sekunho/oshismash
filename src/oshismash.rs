pub mod vtubers;

use axum::response::{IntoResponse, Response};
use hyper::StatusCode;
use serde::Serialize;

use crate::db;

pub enum Error {
    UnableToQuery(tokio_postgres::Error),
    FailedToSetupDb(db::Error),
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
                "E001: Failed when attempting to query the database"
            ),
            Error::FailedToSetupDb(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "E002: Failed to setup database connection",
            ),
        }.into_response()
    }
}
