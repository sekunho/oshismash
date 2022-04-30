use deadpool_postgres::Object;
use tokio_postgres::Row;
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

pub async fn create_guest(client: &Object) -> Result<Guest, oshismash::Error> {
    let row = client.query_one("SELECT * FROM app.create_guest()", &[]).await?;
    let guest = Guest::from(row);

    Ok(guest)
}

pub async fn exists(client: &Object, guest_id: String) -> Result<bool, oshismash::Error> {
    let statement = "SELECT exists(SELECT * FROM app.guests WHERE guest_id = $1 :: UUID)";
    let row = client.query_one(statement, &[&guest_id]).await;

    println!("{:?}", row);

    Ok(true)
}
