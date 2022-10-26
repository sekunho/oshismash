// Web-related stuff
mod oshismash_web;

// "Business logic" of oshismash
mod oshismash;

// DB-related stuff
pub mod db;

pub mod config;

use axum::{routing, Router};
use axum_extra::routing::SpaRouter;
use std::{net::SocketAddr, sync::Arc};
use tower::ServiceBuilder;
use tower_http::add_extension::AddExtensionLayer;

use oshismash_web::handlers;

pub async fn run(config: config::AppConfig, db_handle: db::Handle) -> Result<(), hyper::Error> {
    let db_handle = Arc::new(db_handle);
    let arc_config = Arc::new(config.clone());

    // TODO: Add cookie stuff to middleware
    // TODO: Add rate limiter
    let middleware = ServiceBuilder::new()
        .layer(AddExtensionLayer::new(db_handle))
        .layer(AddExtensionLayer::new(arc_config));

    // TODO: Implement dynamic version of UI (JS)
    let app = Router::new()
        .route("/", routing::get(handlers::vtuber::show_from_cookie))
        .route("/", routing::post(handlers::vote::vote))
        // .route("/api/vtuber/:vtuber_id", handlers::vtuber::results)
        // .route("/rpc/vote", routing::post(handlers::vote::rpc_vote))
        .merge(SpaRouter::new("/assets", "public"))
        .route("/:vtuber_id", routing::get(handlers::vtuber::show_given_id))
        .layer(middleware.into_inner());

    let addr = SocketAddr::from(([0, 0, 0, 0], config.port));

    println!("Running on port {}", config.port);

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
