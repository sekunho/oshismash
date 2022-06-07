// Web-related stuff
mod oshismash_web;

// "Business logic" of oshismash
mod oshismash;

// DB-related stuff
pub mod db;

use axum::{routing, Router};
use axum_extra::routing::SpaRouter;
use std::{net::SocketAddr, sync::Arc};
use tower::ServiceBuilder;
use tower_http::add_extension::AddExtensionLayer;

use oshismash_web::handlers;

pub async fn run(db_handle: db::Handle) -> Result<(), hyper::Error> {
    let db_handle = Arc::new(db_handle);

    // TODO: Add cookie stuff to middleware
    // TODO: Add rate limiter
    let middleware = ServiceBuilder::new().layer(AddExtensionLayer::new(db_handle));

    // TODO: Implement dynamic version of UI (JS)
    let app = Router::new()
        .route("/", routing::get(handlers::vtuber::show_from_cookie))
        .route("/", routing::post(handlers::vote::vote))
        // .route("/rpc/vote", routing::post(handlers::vote::rpc_vote))
        .merge(SpaRouter::new("/assets", "public"))
        // TODO: Remove this for an actual favicon
        .route("/favicon.ico", routing::get(handlers::vtuber::touch_grass))
        .route("/:vtuber_id", routing::get(handlers::vtuber::show_given_id))
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
