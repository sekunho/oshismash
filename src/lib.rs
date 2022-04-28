mod controllers;

// Templates/Markup
mod views;
// "Business logic" of oshismash
mod oshismash;

// DB-related stuff
pub mod db;

use std::{net::SocketAddr, sync::Arc};
use axum::{Router, routing, Extension};

pub async fn run(db_handle: db::Handle) -> Result<(), hyper::Error> {
    let arc_db_handle = Arc::new(db_handle);

    let app = Router::new()
        .route("/", routing::get(controllers::vote::show))
        .route("/vote", routing::post(controllers::vote::vote))
        .route("/rpc/vote/:id", routing::post(controllers::vote::rpc_vote))
        .route("/assets/:name", routing::get(controllers::assets::show))
        .layer(Extension(arc_db_handle));

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
