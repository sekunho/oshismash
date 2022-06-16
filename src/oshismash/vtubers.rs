use serde::{Deserialize, Serialize};
use serde_json::Value;
use tokio_postgres::types::Type;

use super::vote::UserAction;
use crate::oshismash::vote::Stat;

/// `oshismash::vtubers::Error` represents whatever error `oshismash::vtubers`
/// might run into.
#[derive(thiserror::Error, Debug)]
pub enum Error {
    /// If the JSON parsing failed
    #[error("")]
    ValueParseFailed,
    /// Query does not give any data back
    /// Wasn't able to query the DB
    #[error("")]
    FailedToQuery(tokio_postgres::Error),
}

impl From<serde_json::Error> for Error {
    /// Converts `serde_json::Error` to `oshismash::vtubers::Error`.
    fn from(_: serde_json::Error) -> Self {
        // TODO: Maybe make the error more specific?
        Error::ValueParseFailed
    }
}

impl From<tokio_postgres::Error> for Error {
    fn from(e: tokio_postgres::Error) -> Self {
        Error::FailedToQuery(e)
    }
}

/// The resulting intermediary type when parsing the DB's JSON data.
#[derive(Debug, Serialize, Deserialize)]
struct DbStack {
    /// Current VTuber being voted for.
    current: Option<VTuber>,
    /// Results of the previously voted VTuber.
    results: Option<Stat>,
    /// List of VTuber IDs that were voted for.
    voted: Vec<i64>,
    /// Action taken for the current VTuber
    vote_for_current: Option<UserAction>,
}

/// `Stack` is everything that is needed to display things in the UI. There are
/// different outcomes that would yield slightly different UI.
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum Stack {
    /// There's no previous VTuber. Representing the first VTuber entry.
    NoPrev {
        current: VTuber,
        voted: Vec<i64>,
        vote_for_current: Option<UserAction>,
    },
    /// No more VTuber to vote for! Representing no more VTuber entries.
    NoCurrent { prev_result: Stat, voted: Vec<i64> },
    /// Has both a previous and current VTuber entry.
    HasBoth {
        prev_result: Stat,
        current: VTuber,
        voted: Vec<i64>,
        vote_for_current: Option<UserAction>,
    },
}

impl Stack {
    /// Gets the current VTuber. Returns `None` if there's none.
    pub fn get_current(&self) -> Option<&VTuber> {
        match self {
            Stack::NoPrev { current, .. } => Some(current),
            Stack::NoCurrent { .. } => None,
            Stack::HasBoth { current, .. } => Some(current),
        }
    }

    /// Gets the guest's vote list
    pub fn get_vote_list(&self) -> Vec<i64> {
        match self {
            Stack::NoPrev { voted, .. } => voted.to_vec(),
            Stack::NoCurrent { voted, .. } => voted.to_vec(),
            Stack::HasBoth { voted, .. } => voted.to_vec(),
        }
    }

    /// Gets the stats of the previously voted VTuber
    pub fn get_last_voted_stat(&self) -> Option<&Stat> {
        match self {
            Stack::NoPrev { .. } => None,
            Stack::NoCurrent { prev_result, .. } => Some(prev_result),
            Stack::HasBoth { prev_result, .. } => Some(prev_result),
        }
    }

    /// Converts a JSON `Value` to a `Stack`. Uses `DbStack` as an intermediary
    /// type during the conversion.
    pub fn from_value(val: Value) -> Option<Stack> {
        serde_json::from_value(val)
            .ok()
            .and_then(|stack: DbStack| stack.into())
    }
}

impl From<DbStack> for Option<Stack> {
    /// Converts a `DbStack` to an `Option<Stack>`. Results to `None` if both
    /// `current` and `results` are none. This outcome is not possible.
    fn from(value: DbStack) -> Option<Stack> {
        match value {
            DbStack {
                current: None,
                results: None,
                ..
            } => None,

            DbStack {
                current: None,
                results: Some(results),
                voted,
                ..
            } => Some(Stack::NoCurrent {
                prev_result: results,
                voted,
            }),

            DbStack {
                current: Some(current),
                results: Some(prev_result),
                voted,
                vote_for_current,
            } => Some(Stack::HasBoth {
                prev_result,
                current,
                voted,
                vote_for_current,
            }),

            DbStack {
                current: Some(current),
                results: None,
                voted,
                vote_for_current,
            } => Some(Stack::NoPrev {
                current,
                voted,
                vote_for_current,
            }),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct VTuber {
    pub id: i64,
    pub name: String,
    pub description: String,
    pub org_name: String,
    pub next: Option<i64>,
    pub prev: Option<i64>,
    pub img: String,
}

#[derive(Debug)]
pub enum VTuberId {
    /// If there's a current VTuber ID.
    Current(i64),
    /// If there's no current VTuber ID, rely on the previous visited.
    LastVisited(i64),
}

impl VTuberId {
    pub fn get_current(&self) -> Option<i64> {
        match *self {
            VTuberId::Current(id) => Some(id),
            VTuberId::LastVisited(_) => None,
        }
    }
}

pub async fn get_vote_stack(
    client: &deadpool_postgres::Object,
    vtuber_id: &VTuberId,
    guest_id: String,
) -> Result<Stack, Error> {
    let value = match vtuber_id {
        VTuberId::LastVisited(id) => query_vote_stack_from_previous(client, *id, guest_id)
            .await
            .and_then(|row| Ok(row.get::<&str, Value>("get_vote_stack_from_previous"))),

        VTuberId::Current(id) => query_vote_stack_from_current(client, *id, guest_id)
            .await
            .and_then(|row| Ok(row.get::<&str, Value>("get_vote_stack_from_current"))),
    }?;

    println!("{:?}", value);

    Stack::from_value(value).ok_or(Error::ValueParseFailed)
}

async fn query_vote_stack_from_previous(
    client: &deadpool_postgres::Object,
    prev_vtuber_id: i64,
    guest_id: String,
) -> Result<tokio_postgres::Row, tokio_postgres::Error> {
    let statement = client
        .prepare_typed(
            "SELECT * FROM app.get_vote_stack_from_previous($1::BIGINT, $2::UUID)",
            &[Type::INT8, Type::TEXT],
        )
        .await?;

    client
        .query_one(&statement, &[&prev_vtuber_id, &guest_id])
        .await
}

async fn query_vote_stack_from_current(
    client: &deadpool_postgres::Object,
    current_vtuber_id: i64,
    guest_id: String,
) -> Result<tokio_postgres::Row, tokio_postgres::Error> {
    let statement = client
        .prepare_typed(
            "SELECT * FROM app.get_vote_stack_from_current($1, $2::UUID)",
            &[Type::INT8, Type::TEXT],
        )
        .await?;

    client
        .query_one(&statement, &[&current_vtuber_id, &guest_id])
        .await
}

#[cfg(test)]
mod tests {
    use serde_json::{json, Value};

    use super::Stack;
    use crate::oshismash::{
        vote::{Stat, UserAction},
        vtubers::VTuber,
    };

    fn mock_has_current_no_prev() -> Value {
        json!({
            "current": {
                "description": "A weirdo",
                "id": 1,
                "img": "https://www.vshojo.com/wp-content/uploads/nyanners-full_solo.png",
                "name": "Nyatasha Nyanners",
                "next": 2,
                "org_name": "VShojo",
                "prev": Value::Null,
            },
            "results": Value::Null,
            "vote_for_current": "smashed",
            "voted": json!([1, 2, 3, 4])
        })
    }

    fn mock_has_none_in_both() -> Value {
        json!({
            "current": Value::Null,
            "results": Value::Null,
            "vote_for_current": "smashed",
            "voted": [1, 2, 3, 4]
        })
    }

    fn mock_has_prev_no_current() -> Value {
        json!({
            "current": Value::Null,
            "results": {
                "vtuber_id": 2,
                "name": "Veibae",
                "description": "A weirdo",
                "img": "https://www.vshojo.com/wp-content/uploads/nyanners-full_solo.png",
                "next": 3,
                "prev": 1,
                "smashes": 4,
                "passes": 1,
            },
            "vote_for_current": "smashed",
            "voted": [1, 2, 3, 4]
        })
    }

    fn mock_has_both() -> Value {
        json!({
            "current": {
                "description": "A weirdo",
                "id": 1,
                "img": "https://www.vshojo.com/wp-content/uploads/nyanners-full_solo.png",
                "name": "Nyatasha Nyanners",
                "next": 2,
                "org_name": "VShojo",
                "prev": Value::Null,
            },
            "results": {
                "vtuber_id": 2,
                "name": "Veibae",
                "description": "A weirdo",
                "img": "https://www.vshojo.com/wp-content/uploads/nyanners-full_solo.png",
                "next": 3,
                "prev": 1,
                "smashes": 4,
                "passes": 1,
            },
            "vote_for_current": "smashed",
            "voted": [1, 2, 3, 4]
        })
    }

    #[test]
    fn parse_has_current_no_prev() {
        let found = Stack::from_value(mock_has_current_no_prev());
        let voted_ids: Vec<i64> = Vec::from([1, 2, 3, 4]);

        assert_eq!(found.clone().unwrap().get_vote_list(), voted_ids);

        assert_eq!(
            found.clone().unwrap(),
            Stack::NoPrev {
                current: VTuber {
                    id: 1,
                    name: "Nyatasha Nyanners".to_string(),
                    description: "A weirdo".to_string(),
                    org_name: "VShojo".to_string(),
                    next: Some(2),
                    prev: None,
                    img: "https://www.vshojo.com/wp-content/uploads/nyanners-full_solo.png"
                        .to_string(),
                },
                voted: voted_ids,
                vote_for_current: Some(UserAction::Smashed)
            }
        );
    }

    #[test]
    fn parse_has_prev_no_current() {
        let found = Stack::from_value(mock_has_prev_no_current());
        let voted_ids: Vec<i64> = Vec::from([1, 2, 3, 4]);

        assert_eq!(found.clone().unwrap().get_vote_list(), voted_ids);

        assert_eq!(
            found.clone().unwrap(),
            Stack::NoCurrent {
                prev_result: Stat {
                    vtuber_id: 2,
                    name: "Veibae".to_string(),
                    img: Some(
                        "https://www.vshojo.com/wp-content/uploads/nyanners-full_solo.png"
                            .to_string()
                    ),
                    smashes: 4,
                    passes: 1,
                },
                voted: voted_ids,
            }
        );
    }

    #[test]
    fn parse_has_both() {
        let found = Stack::from_value(mock_has_both());
        let voted_ids: Vec<i64> = Vec::from([1, 2, 3, 4]);

        assert_eq!(found.clone().unwrap().get_vote_list(), voted_ids);

        assert_eq!(
            found.clone().unwrap(),
            Stack::HasBoth {
                prev_result: Stat {
                    vtuber_id: 2,
                    name: "Veibae".to_string(),
                    img: Some(
                        "https://www.vshojo.com/wp-content/uploads/nyanners-full_solo.png"
                            .to_string()
                    ),
                    smashes: 4,
                    passes: 1,
                },
                current: VTuber {
                    id: 1,
                    name: "Nyatasha Nyanners".to_string(),
                    description: "A weirdo".to_string(),
                    org_name: "VShojo".to_string(),
                    next: Some(2),
                    prev: None,
                    img: "https://www.vshojo.com/wp-content/uploads/nyanners-full_solo.png"
                        .to_string(),
                },
                voted: voted_ids,
                vote_for_current: Some(UserAction::Smashed)
            }
        );
    }

    #[test]
    fn parsing_null_in_both_fails() {
        let found = Stack::from_value(mock_has_none_in_both());

        assert!(found.clone().is_none());
    }
}
