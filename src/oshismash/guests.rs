use deadpool_postgres::Object;
use tokio_postgres::{Row, types::Type};
use crate::oshismash;

#[derive(Debug)]
pub struct Guest {
    pub guest_id: String,
}

impl From<Row> for Guest {
    fn from(row: Row) -> Self {
        Guest {
            guest_id: row.get("guest_id"),
        }
    }
}

/// Creates an anonymous guest
pub async fn create_guest(client: &Object) -> Result<Guest, oshismash::Error> {
    let statement = "SELECT * FROM app.create_guest()";
    let statement = client.prepare_typed(statement, &[]).await?;
    let row = client.query_one(&statement, &[]).await?;
    let guest = Guest::from(row);

    Ok(guest)
}

/// Checks if the guest token is valid (if it exists in the DB).
pub async fn is_valid(client: &Object, guest_id: &str) -> Result<bool, oshismash::Error> {
    let statement = "SELECT exists(SELECT * FROM app.guests WHERE guest_id = $1 :: UUID)";
    let statement = client.prepare_typed(statement, &[Type::TEXT]).await?;

    let is_valid: bool = client
        .query_one(&statement, &[&guest_id])
        .await
        .map_err(|e| {
            match e.code() {
                Some(other_e) => match other_e.code() {
                    "22P02" => oshismash::Error::InvalidGuest,
                    _       => oshismash::Error::UnableToQuery(e)
                }
                None => oshismash::Error::UnableToQuery(e)
            }
        })?
        .get("exists");

    Ok(is_valid)
}
