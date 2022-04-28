#![forbid(unsafe_code)]

use oshismash::db;

#[tokio::main]
async fn main() {
    match db::Handle::from_config() {
        Ok(db_handle) => {
            if let Err(e) = oshismash::run(db_handle).await {
                eprintln!("{}", e);
            };
        }

        Err(_) => todo!(),
    }

}
