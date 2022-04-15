#![forbid(unsafe_code)]

#[tokio::main]
async fn main() {
    if let Err(e) = oshismash::run().await {
        eprintln!("{}", e);
    };
}
