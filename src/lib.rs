mod controllers;

use std::net::SocketAddr;
use axum::{Router, routing};

pub async fn run() -> Result<(), hyper::Error> {
    let app = Router::new()
        .route("/", routing::get(controllers::test));

    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));

    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .with_graceful_shutdown(signal_shutdown())
        .await
}

async fn signal_shutdown() {
    tokio::signal::ctrl_c()
        .await
        .expect("expect tokio signal ctrl-c");

   println!("signal shutdown")
}
