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
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum Stack {
    NoPrev { current: VTuber },
    NoCurrent { prev_result: Stat },
    HasBoth { prev_result: Stat, current: VTuber },
}

impl Stack {
    pub fn get_current(&self) -> Option<&VTuber> {
        match self {
            Stack::NoPrev { current } => Some(current),
            Stack::NoCurrent { prev_result: _ } => None,
            Stack::HasBoth {
                prev_result: _,
                current,
            } => Some(current),
        }
    }

    pub fn get_last_voted_stat(&self) -> Option<&Stat> {
        match self {
            Stack::NoPrev { current: _ } => None,
            Stack::NoCurrent { prev_result } => Some(prev_result),
            Stack::HasBoth {
                prev_result,
                current: _,
            } => Some(prev_result),
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
            } => Err(Error::NoData),

            DbStack {
                current: None,
                results: Some(results),
            } => Ok(Stack::NoCurrent {
                prev_result: results,
            }),

            DbStack {
                current: Some(current),
                results: Some(prev_result),
            } => Ok(Stack::HasBoth {
                prev_result,
                current,
            }),

            DbStack {
                current: Some(current),
                results: None,
            } => Ok(Stack::NoPrev { current }),
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

pub async fn get_vote_stack(
    client: &deadpool_postgres::Object,
    last_voted_vtuber_id: Option<i64>,
) -> Result<Stack, Error> {
    match last_voted_vtuber_id {
        Some(prev_id) => query_vote_stack_from_previous(client, prev_id).await,
        None => query_vote_stack_from_current(client, 1).await,
    }
}

async fn query_vote_stack_from_previous(
    client: &deadpool_postgres::Object,
    prev_vtuber_id: i64,
) -> Result<Stack, Error> {
    let statement = client
        .prepare_typed(
            "SELECT * FROM app.get_vote_stack_from_previous($1::BIGINT)",
            &[Type::INT8]
        )
        .await
        .map_err(|e| {
            println!("{:?}", e);
            Error::FailedToPrepareStatement
        })?;

    let val: Option<Value> = client
        .query_one(&statement, &[&prev_vtuber_id])
        .await
        .map_err(|_| Error::FailedToQuery)?
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
) -> Result<Stack, Error> {
    let statement = client
        .prepare_typed(
            "SELECT * FROM app.get_vote_stack_from_current($1)",
            &[Type::INT8]
        )
        .await
        .map_err(|e| {
            println!("{:?}", e);
            Error::FailedToPrepareStatement
        })?;

    let val: Option<Value> = client
        .query_one(&statement, &[&current_vtuber_id])
        .await
        .map_err(|_| Error::FailedToQuery)?
        .get("get_vote_stack_from_current");

    match val {
        Some(val) => Stack::from_value(val).map_err(|e| {
            println!("{:?}", e);
            e
        }),
        None => Err(Error::FailedToParseValue),
    }
}
