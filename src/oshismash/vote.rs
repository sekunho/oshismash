use axum::response::IntoResponse;
use hyper::StatusCode;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tokio_postgres::types::Type;
use uuid::Uuid;

use super::vtubers::Stack;

// TODO: Implement error
#[derive(thiserror::Error, Debug, PartialEq)]
pub enum ParseError {
    #[error("no such user vote action `{0}`")]
    InvalidAction(String),
    #[error("`{0}` has invalid format (expected integer)")]
    InvalidVtuberIdFormat(String),
    #[error("`{0}` has invalid format (expected UUID v4)")]
    InvalidGuestIdFormat(String),
    #[error("`{0}` field not found but is required")]
    MissingField(String),
}

impl IntoResponse for ParseError {
    fn into_response(self) -> axum::response::Response {
        // TODO: Move this out to `oshismash_web`
        // TODO: Replace the text with an HTML template
        let res = match self {
            ParseError::InvalidAction(_action) => (
                StatusCode::BAD_REQUEST,
                "Replace this with an actual HTML template",
            ),
            ParseError::MissingField(_) => (
                StatusCode::BAD_REQUEST,
                "Replace this with an actual HTML template",
            ),
            ParseError::InvalidVtuberIdFormat(_) => (
                StatusCode::BAD_REQUEST,
                "Replace this with an actual HTML template",
            ),
            ParseError::InvalidGuestIdFormat(_) => (
                StatusCode::BAD_REQUEST,
                "Replace this with an actual HTML template",
            ),
        };

        res.into_response()
    }
}

#[derive(thiserror::Error, Debug)]
pub enum VoteError {
    #[error("failed to prepare vote query ({0})")]
    QueryPrepFailed(tokio_postgres::Error),
    #[error("unable to run the vote query ({0})")]
    QueryFailed(tokio_postgres::Error),
    #[error("the returning `Value` is not a valid `Stack`")]
    InvalidDbValue,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub enum UserAction {
    #[serde(rename = "smashed")]
    Smashed,
    #[serde(rename = "passed")]
    Passed,
}

impl UserAction {
    fn from(action: &str) -> Option<UserAction> {
        match action {
            "smashed" => Some(UserAction::Smashed),
            "passed" => Some(UserAction::Passed),
            _ => None,
        }
    }
}

// TODO(sekun): Rename to `Ballot`?
#[derive(Debug, PartialEq, Clone)]
pub struct Vote {
    pub vtuber_id: i64,
    pub guest_id: Uuid,
    pub action: UserAction,
}

impl Vote {
    pub fn from(val: Value) -> Result<Vote, ParseError> {
        let vtuber_id = val.get("vtuber_id").and_then(|a| a.as_str());

        let vtuber_id = match vtuber_id {
            Some(vtuber_id) => vtuber_id
                .parse::<i64>()
                .map_err(|_| ParseError::InvalidVtuberIdFormat(String::from(vtuber_id))),
            None => Err(ParseError::MissingField(String::from("vtuber_id"))),
        }?;

        let guest_id = val.get("guest_id").and_then(|a| a.as_str());
        let action = val.get("action").and_then(|a| a.as_str());

        match (guest_id, action) {
            (Some(guest_id), Some(action)) => {
                let action = UserAction::from(action).map_or_else(
                    || Err(ParseError::InvalidAction(String::from(action))),
                    |a| Ok(a),
                )?;

                let guest_id = Uuid::parse_str(guest_id)
                    .map_err(|_| ParseError::InvalidGuestIdFormat(String::from(guest_id)))?;

                let vote = Vote {
                    vtuber_id,
                    guest_id,
                    action,
                };

                Ok(vote)
            }

            (None, _) => Err(ParseError::MissingField(String::from("guest_id"))),
            (_, None) => Err(ParseError::MissingField(String::from("action"))),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
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
) -> Result<Stack, VoteError> {
    let action = match vote_entry.action {
        UserAction::Smashed => "smashed",
        UserAction::Passed => "passed",
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
            VoteError::QueryPrepFailed(e)
        })?;

    let val: Value = client
        .query_one(
            &vote_statement,
            &[
                &vote_entry.guest_id.to_string(),
                &vote_entry.vtuber_id,
                &action,
            ],
        )
        .await
        .map_err(|e| {
            println!("{e}");
            VoteError::QueryFailed(e)
        })?
        .get("vote");

    println!("{:?}", val);

    let stack = Stack::from_value(val).ok_or(VoteError::InvalidDbValue)?;

    Ok(stack)
}

#[cfg(test)]
mod tests {
    use super::{ParseError, UserAction, Vote};
    use serde_json::json;
    use uuid::Uuid;

    #[test]
    fn valid_json_vote() {
        let guest_id = Uuid::parse_str("0b76fdde-9910-402d-b7c2-97c02247b5fd").unwrap();
        let vote_passed = json!({
            "vtuber_id": "1",
            "guest_id": "0b76fdde-9910-402d-b7c2-97c02247b5fd",
            "action": "passed"
        });

        let vote_smashed = json!({
            "vtuber_id": "1",
            "guest_id": "0b76fdde-9910-402d-b7c2-97c02247b5fd",
            "action": "smashed"
        });

        let expected_passed = Vote {
            vtuber_id: 1,
            guest_id,
            action: UserAction::Passed,
        };

        let expected_smashed = Vote {
            vtuber_id: 1,
            guest_id,
            action: UserAction::Smashed,
        };

        assert!(Vote::from(vote_passed).unwrap() == expected_passed);
        assert!(Vote::from(vote_smashed).unwrap() == expected_smashed);
    }

    #[test]
    fn mispelled_vote_action() {
        // Mispelled vote action
        let vote_a = json!({
            "vtuber_id": "1",
            "guest_id": "0b76fdde-9910-402d-b7c2-97c02247b5fd",
            "action": "smashe"
        });

        let vote_b = json!({
            "vtuber_id": "1",
            "guest_id": "0b76fdde-9910-402d-b7c2-97c02247b5fd",
            "action": "pasxed"
        });

        assert!(Vote::from(vote_a) == Err(ParseError::InvalidAction("smashe".to_string())));
        assert!(Vote::from(vote_b) == Err(ParseError::InvalidAction("pasxed".to_string())));
    }

    #[test]
    fn missing_fields_in_vote() {
        let vote_with_missing_action = json!({
            "vtuber_id": "1",
            "guest_id": "0b76fdde-9910-402d-b7c2-97c02247b5fd",
        });

        let vote_with_missing_vtuber_id = json!({
            "guest_id": "0b76fdde-9910-402d-b7c2-97c02247b5fd",
            "action": "smashed"
        });

        let vote_with_missing_guest_id = json!({
            "vtuber_id": "1",
            "action": "smashed"
        });

        assert!(
            Vote::from(vote_with_missing_action)
                == Err(ParseError::MissingField("action".to_string()))
        );
        assert!(
            Vote::from(vote_with_missing_vtuber_id)
                == Err(ParseError::MissingField("vtuber_id".to_string()))
        );
        assert!(
            Vote::from(vote_with_missing_guest_id)
                == Err(ParseError::MissingField("guest_id".to_string()))
        );
    }
}
