use std::sync::Arc;

use axum::{
    async_trait,
    extract::{Form, FromRequest},
    response::IntoResponse,
    BoxError, Extension,
};
use axum_extra::extract::cookie::CookieJar;
use hyper::StatusCode;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tokio_postgres::types::Type;

use super::{guests::GuestId, vtubers::Stack};
use crate::{
    db,
    oshismash::{self, guests},
};

// TODO: Implement error
#[derive(Debug)]
pub enum Error {
    InvalidVote,
    InvalidAction,
    MissingGuestId,
}

impl IntoResponse for Error {
    fn into_response(self) -> axum::response::Response {
        let res = match self {
            Error::InvalidVote => (
                StatusCode::BAD_REQUEST,
                "Replace this with actual HTML template",
            ),
            Error::InvalidAction => (
                StatusCode::BAD_REQUEST,
                "Replace this with an actual HTML template",
            ),
            Error::MissingGuestId => (
                StatusCode::BAD_REQUEST,
                "replace this with an actual HTML template",
            ),
        };

        res.into_response()
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub enum Action {
    #[serde(rename = "smashed")]
    Smashed,
    #[serde(rename = "passed")]
    Passed,
}

impl Action {
    fn from(action: &str) -> Option<Action> {
        match action {
            "smashed" => Some(Action::Smashed),
            "passed" => Some(Action::Passed),
            _ => None,
        }
    }
}

// TODO(sekun): Rename to `Ballot`?
#[derive(Debug, PartialEq, Clone)]
pub struct Vote {
    pub vtuber_id: i64,
    pub guest_id: String,
    pub action: Action,
}

impl Vote {
    pub fn from(val: Value) -> Result<Vote, Error> {
        let vtuber_id = val
            .get("vtuber_id")
            .and_then(|a| a.as_str())
            .and_then(|a| a.parse::<i64>().ok());

        let guest_id = val.get("guest_id").and_then(|a| a.as_str());
        let action = val.get("action").and_then(|a| a.as_str());

        match (vtuber_id, guest_id, action) {
            (Some(vtuber_id), Some(guest_id), Some(action)) => {
                let action =
                    Action::from(action).map_or_else(|| Err(Error::InvalidAction), |a| Ok(a))?;

                let vote = Vote {
                    vtuber_id,
                    guest_id: guest_id.to_string(),
                    action,
                };

                Ok(vote)
            }

            _ => Err(Error::InvalidVote),
        }
    }
}

#[async_trait]
impl<B> FromRequest<B> for Vote
where
    // Copied these trait bounds from
    // https://docs.rs/axum/latest/axum/extract/struct.Form.html#impl-FromRequest%3CB%3E
    B: Send + axum::body::HttpBody,
    B::Data: Send,
    B::Error: Into<BoxError>,
{
    type Rejection = oshismash::Error;

    async fn from_request(
        req: &mut axum::extract::RequestParts<B>,
    ) -> Result<Self, Self::Rejection> {
        let form_data = req.extract::<Form<Value>>().await;

        let guest_id = req
            .extract::<CookieJar>()
            .await
            .map_err(|_| Error::MissingGuestId)
            .and_then(|jar| {
                jar.get("id")
                    .and_then(|cookie| Some(GuestId(cookie.value().to_string())))
                    .ok_or(Error::MissingGuestId)
            });

        let db = req.extract::<Extension<Arc<db::Handle>>>().await;

        // Hadouken'd :(
        match (form_data, guest_id, db) {
            (Ok(Form(Value::Object(mut form_data))), Ok(GuestId(guest_id)), Ok(db)) => {
                match db.get_client().await {
                    Ok(client) => match guests::is_valid(&client, guest_id.as_str()).await {
                        Ok(true) => {
                            form_data.insert(String::from("guest_id"), Value::String(guest_id));

                            let result = Vote::from(Value::Object(form_data))?;
                            Ok(result)
                        }
                        Ok(false) => Err(oshismash::Error::InvalidGuest),
                        Err(e) => Err(e),
                    },
                    Err(e) => Err(oshismash::Error::from(e)),
                }
            }
            (_, _, _) => Err(oshismash::Error::FailedToParseVoteEntry),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Stat {
    pub vtuber_id: i64,
    pub name: String,
    pub img: Option<String>,
    pub smashes: i64,
    pub passes: i64,
}

/// Votes for a VTuber
pub async fn vote(
    client: &deadpool_postgres::Object,
    vote_entry: Vote,
) -> Result<Stack, oshismash::Error> {
    let action = match vote_entry.action {
        Action::Smashed => "smashed",
        Action::Passed => "passed",
    };

    println!("Vote Entry: {:?}", vote_entry);

    let vote_statement = client
        .prepare_typed(
            "SELECT * FROM app.vote($1 :: UUID, $2 :: BIGINT, $3 :: app.ACTION)",
            &[Type::TEXT, Type::INT8, Type::TEXT],
        )
        .await
        .map_err(|e| {
            println!("{e}");
            oshismash::Error::from(e)
        })?;

    let val: Value = client
        .query_one(
            &vote_statement,
            &[&vote_entry.guest_id, &vote_entry.vtuber_id, &action],
        )
        .await
        .map_err(|e| {
            println!("{e}");
            oshismash::Error::from(e)
        })?
        .get("vote");

    println!("{:?}", val);

    let stack = Stack::from_value(val).ok_or(oshismash::Error::FailedToParseStack(
        oshismash::vtubers::Error::ValueParseFailed,
    ))?;

    Ok(stack)
}

#[cfg(test)]
mod tests {
    use super::{Action, Vote};
    use serde_json::json;

    #[test]
    fn valid_json_vote() {
        let vote = json!({
            "vtuber_id": "1",
            "guest_id": "0b76fdde-9910-402d-b7c2-97c02247b5fd",
            "action": "passed"
        });

        let expected = Vote {
            vtuber_id: 1,
            guest_id: "0b76fdde-9910-402d-b7c2-97c02247b5fd".to_string(),
            action: Action::Passed,
        };

        assert!(Vote::from(vote).unwrap() == expected);
    }
}
