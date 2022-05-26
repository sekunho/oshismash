mod component;
mod controllers;

// Templates/Markup
mod views;

// "Business logic" of oshismash
mod oshismash;

// DB-related stuff
pub mod db;

use axum::{routing, Router};
use std::{net::SocketAddr, sync::Arc};
use tower::ServiceBuilder;
use tower_http::add_extension::AddExtensionLayer;

pub async fn run(db_handle: db::Handle) -> Result<(), hyper::Error> {
    let db_handle = Arc::new(db_handle);

    // TODO: Add cookie stuff to middleware
    let middleware = ServiceBuilder::new().layer(AddExtensionLayer::new(db_handle));

    let app = Router::new()
        .route("/", routing::get(controllers::vote::index))
        .route("/", routing::post(controllers::vote::vote))
        // .route("/rpc/vote", routing::post(controllers::vote::rpc_vote))
        .route("/assets/:name", routing::get(controllers::assets::show))
        .layer(middleware.into_inner());

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
