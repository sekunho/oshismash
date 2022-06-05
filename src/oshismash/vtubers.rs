use serde::{Deserialize, Serialize};
use serde_json::Value;
use tokio_postgres::types::Type;

use crate::oshismash::vote::Stat;

/// `oshismash::vtubers::Error` represents whatever error `oshismash::vtubers`
/// might run into.
#[derive(Debug)]
pub enum Error {
    /// If the JSON parsing failed
    ValueParseFailed,
    /// Query does not give any data back
    /// Wasn't able to query the DB
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
pub struct DbStack {
    /// Current VTuber being voted for.
    current: Option<VTuber>,
    /// Results of the previously voted VTuber.
    results: Option<Stat>,
    /// List of VTuber IDs that were voted for.
    voted: Vec<i64>,
}

/// `Stack` is everything that is needed to display things in the UI. There are
/// different outcomes that would yield slightly different UI.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum Stack {
    /// There's no previous VTuber. Representing the first VTuber entry.
    NoPrev { current: VTuber, voted: Vec<i64> },
    /// No more VTuber to vote for! Representing no more VTuber entries.
    NoCurrent { prev_result: Stat, voted: Vec<i64> },
    /// Has both a previous and current VTuber entry.
    HasBoth {
        prev_result: Stat,
        current: VTuber,
        voted: Vec<i64>,
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
                voted: _,
            } => None,

            DbStack {
                current: None,
                results: Some(results),
                voted,
            } => Some(Stack::NoCurrent {
                prev_result: results,
                voted,
            }),

            DbStack {
                current: Some(current),
                results: Some(prev_result),
                voted,
            } => Some(Stack::HasBoth {
                prev_result,
                current,
                voted,
            }),

            DbStack {
                current: Some(current),
                results: None,
                voted,
            } => Some(Stack::NoPrev { current, voted }),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
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
        VTuberId::LastVisited(id) => {
            query_vote_stack_from_previous(client, *id, guest_id)
                .await
                .and_then(|row| {
                    Ok(row.get::<&str, Value>("get_vote_stack_from_previous"))
                })
        }

        VTuberId::Current(id) => {
            query_vote_stack_from_current(client, *id, guest_id)
                .await
                .and_then(|row| {
                    Ok(row.get::<&str, Value>("get_vote_stack_from_current"))
                })
        }
    }?;

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
