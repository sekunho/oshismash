use serde::{Deserialize, Serialize};
use serde_json::Value;
use tokio_postgres::types::Type;

use super::vtubers::Stack;
use crate::oshismash;

// TODO: Implement error
#[derive(Debug)]
pub enum Error {
    InvalidVote,
    InvalidAction,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub enum Action {
    Smashed,
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

#[derive(Debug, PartialEq)]
pub struct Vote {
    pub vtuber_id: String,
    pub guest_id: String,
    pub action: Action,
}

impl Vote {
    pub fn from(val: Value) -> Result<Vote, Error> {
        let vtuber_id = val.get("vtuber_id").and_then(|a| a.as_str());
        let guest_id = val.get("guest_id").and_then(|a| a.as_str());
        let action = val.get("action").and_then(|a| a.as_str());

        match (vtuber_id, guest_id, action) {
            (Some(vtuber_id), Some(guest_id), Some(action)) => {
                let action =
                    Action::from(action).map_or_else(|| Err(Error::InvalidAction), |a| Ok(a))?;

                let vote = Vote {
                    vtuber_id: vtuber_id.to_string(),
                    guest_id: guest_id.to_string(),
                    action,
                };

                Ok(vote)
            }

            _ => Err(Error::InvalidVote),
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
            &[Type::TEXT, Type::TEXT, Type::TEXT],
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

    Ok(Stack::from_value(val)?)
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
            vtuber_id: "1".to_string(),
            guest_id: "0b76fdde-9910-402d-b7c2-97c02247b5fd".to_string(),
            action: Action::Passed,
        };

        assert!(Vote::from(vote).unwrap() == expected);
    }
}
