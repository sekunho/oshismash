use serde::{Deserialize, Serialize};
use serde_json::Value;
use tokio_postgres::types::Type;

use crate::oshismash::vote::Stat;

#[derive(Debug)]
pub enum Error {
    ParseFailed,
    NoData,
    FailedToPrepareStatement,
    FailedToQuery,
    FailedToParseValue,
}

impl From<serde_json::Error> for Error {
    fn from(_: serde_json::Error) -> Self {
        // TODO: Maybe make the error more specific?
        Error::ParseFailed
    }
}

/// The resulting intermediary type when parsing the DB's JSON data.
#[derive(Debug, Serialize, Deserialize)]
pub struct DbStack {
    current: Option<VTuber>,
    results: Option<Stat>,
    voted: Vec<i64>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum Stack {
    NoPrev {
        current: VTuber,
        voted: Vec<i64>,
    },
    NoCurrent {
        prev_result: Stat,
        voted: Vec<i64>,
    },
    HasBoth {
        prev_result: Stat,
        current: VTuber,
        voted: Vec<i64>,
    },
}

impl Stack {
    pub fn get_current(&self) -> Option<&VTuber> {
        match self {
            Stack::NoPrev { current, .. } => Some(current),
            Stack::NoCurrent { .. } => None,
            Stack::HasBoth { current, .. } => Some(current),
        }
    }

    pub fn get_vote_list(&self) -> Vec<i64> {
        match self {
            Stack::NoPrev { voted, .. } => voted.to_vec(),
            Stack::NoCurrent { voted, .. } => voted.to_vec(),
            Stack::HasBoth { voted, .. } => voted.to_vec(),
        }
    }

    pub fn get_last_voted_stat(&self) -> Option<&Stat> {
        match self {
            Stack::NoPrev { .. } => None,
            Stack::NoCurrent { prev_result, .. } => Some(prev_result),
            Stack::HasBoth { prev_result, .. } => Some(prev_result),
        }
    }

    pub fn from_value(val: Value) -> Result<Stack, Error> {
        let stack: DbStack = serde_json::from_value(val).map_err(|e| {
            println!("{:?}", e);
            e
        })?;

        Stack::from_db_stack(stack)
    }

    pub fn from_db_stack(db_stack: DbStack) -> Result<Stack, Error> {
        db_stack.try_into().map_err(|e| {
            println!("{:?}", e);
            e
        })
    }
}

impl TryFrom<DbStack> for Stack {
    type Error = Error;

    fn try_from(value: DbStack) -> Result<Self, Self::Error> {
        match value {
            DbStack {
                current: None,
                results: None,
                voted: _,
            } => Err(Error::NoData),

            DbStack {
                current: None,
                results: Some(results),
                voted,
            } => Ok(Stack::NoCurrent {
                prev_result: results,
                voted,
            }),

            DbStack {
                current: Some(current),
                results: Some(prev_result),
                voted,
            } => Ok(Stack::HasBoth {
                prev_result,
                current,
                voted,
            }),

            DbStack {
                current: Some(current),
                results: None,
                voted,
            } => Ok(Stack::NoPrev { current, voted }),
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
    match vtuber_id {
        VTuberId::LastVisited(id) => query_vote_stack_from_previous(client, *id, guest_id).await,
        VTuberId::Current(id) => query_vote_stack_from_current(client, *id, guest_id).await,
    }
}

async fn query_vote_stack_from_previous(
    client: &deadpool_postgres::Object,
    prev_vtuber_id: i64,
    guest_id: String,
) -> Result<Stack, Error> {
    let statement = client
        .prepare_typed(
            "SELECT * FROM app.get_vote_stack_from_previous($1::BIGINT, $2::UUID)",
            &[Type::INT8, Type::TEXT],
        )
        .await
        .map_err(|e| {
            println!("{:?}", e);
            Error::FailedToPrepareStatement
        })?;

    let val: Option<Value> = client
        .query_one(&statement, &[&prev_vtuber_id, &guest_id])
        .await
        .map_err(|e| {
            println!("{:?}", e);
            Error::FailedToQuery
        })?
        .get("get_vote_stack_from_previous");

    match val {
        Some(val) => Stack::from_value(val).map_err(|e| {
            println!("{:?}", e);
            e
        }),
        None => Err(Error::FailedToParseValue),
    }
}

async fn query_vote_stack_from_current(
    client: &deadpool_postgres::Object,
    current_vtuber_id: i64,
    guest_id: String,
) -> Result<Stack, Error> {
    let statement = client
        .prepare_typed(
            "SELECT * FROM app.get_vote_stack_from_current($1, $2::UUID)",
            &[Type::INT8, Type::TEXT],
        )
        .await
        .map_err(|e| {
            println!("{:?}", e);
            Error::FailedToPrepareStatement
        })?;

    let val: Option<Value> = client
        .query_one(&statement, &[&current_vtuber_id, &guest_id])
        .await
        .map_err(|e| {
            println!("{:?}", e);
            Error::FailedToQuery
        })?
        .get("get_vote_stack_from_current");

    match val {
        Some(val) => Stack::from_value(val).map_err(|e| {
            println!("{:?}", e);
            e
        }),
        None => Err(Error::FailedToParseValue),
    }
}
