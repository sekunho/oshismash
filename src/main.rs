#![forbid(unsafe_code)]

#[tokio::main]
async fn main() {
    if let Err(e) = vtubersmash::run().await {
        eprintln!("{}", e);
    };
}
