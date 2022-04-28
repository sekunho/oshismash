use deadpool_postgres::Object;
use tokio_postgres::Row;
use tokio_postgres::types::Type;
use serde::{Serialize, Deserialize};

use crate::oshismash;

#[derive(Serialize, Deserialize, Debug)]
pub struct VoteEntry {
    pub vtuber_id: String,
    pub action: Action,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct VTuber {
    pub id: i64,
    pub name: String,
    pub description: String,
    pub smashes: i64,
    pub passes: i64,
}

#[derive(Debug, Serialize)]
pub struct CardStack {
    /// The VTuber that was previously voted.
    pub prev: Option<VTuber>,

    /// The current one in display.
    pub current: Option<VTuber>,

    /// Next VTuber
    pub next: Option<VTuber>,
}

impl From<Row> for VTuber {
    fn from(row: Row) -> VTuber {
        VTuber {
            id: row.get("vtuber_id"),
            name: row.get("name"),
            description: row.get("description"),
            smashes: row.get("smashes"),
            passes: row.get("passes"),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Action {
    Smashed,
    Passed,
}

pub async fn vote(client: &Object, vote_entry: VoteEntry) -> Result<CardStack, oshismash::Error> {
    let action = match vote_entry.action {
        Action::Smashed => "smashed",
        Action::Passed => "passed",
    };

    let vote_statement = client.prepare_typed(
        "SELECT * FROM app.vote($1 :: BIGINT, $2 :: app.ACTION)",
        &[Type::TEXT, Type::TEXT]
    ).await?;

    let prev = VTuber::from(
        client.query_one(&vote_statement,
        &[&vote_entry.vtuber_id, &action]).await?
    );

    let card_stack = CardStack {
        prev: Some(prev),
        current: None,
        next: None
    };

    Ok(card_stack)
}
