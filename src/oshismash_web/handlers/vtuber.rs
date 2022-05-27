use std::sync::Arc;

use axum::Extension;
use axum_extra::extract::cookie;
use hyper::{header::LOCATION, HeaderMap, StatusCode};
use maud::{html, Markup};

use crate::db;
use crate::oshismash_web::client_data::ClientData;
use crate::{
    oshismash::{
        self,
        vtubers::{self, Stack, VTuberId},
    },
    oshismash_web::cookie_util,
};

/// Main page for the smash or pass
pub async fn show_from_cookie(
    jar: cookie::CookieJar,
    client_data: ClientData,
) -> Result<(StatusCode, HeaderMap, cookie::CookieJar), oshismash::Error> {
    // NOTE: Am I supposed to move the cookie stuff to `tower`/middleware?
    // Cookies:
    // https://developer.mozilla.org/en-US/docs/Web/HTTP/Cookies
    let jar = match client_data.vtuber_id {
        VTuberId::Current(id) => jar
            .add(cookie_util::create("current", id))
            .add(cookie_util::create("last_visited", "none")),

        VTuberId::LastVisited(id) => jar
            .add(cookie_util::create("last_visited", id))
            .add(cookie_util::create("current", "none")),
    }
    .add(cookie_util::create("id", client_data.guest_id));

    let mut headers = HeaderMap::new();

    // TODO: Replace placeholder URL with actual host
    let url = match client_data.vtuber_id {
        VTuberId::Current(id) => format!("http://localhost:3000/{}", id),
        VTuberId::LastVisited(_) => "http://localhost:3000/touch-grass".to_string()
    };

    headers.insert(LOCATION, url.parse().unwrap());

    Ok((StatusCode::FOUND, headers, jar))
}

pub async fn show_given_id(
    Extension(db_handle): Extension<Arc<db::Handle>>,
    client_data: ClientData,
    jar: cookie::CookieJar,
) -> Result<(cookie::CookieJar, Stack), oshismash::Error> {
    println!("From ID: {:?}", client_data);
    let client = db_handle.pool.get().await?;

    let stack = vtubers::get_vote_stack(&client, &client_data.vtuber_id)
        .await
        .map_err(|e| {
            println!("{:?}", e);
            e
        })?;

    let jar = match client_data.vtuber_id {
        VTuberId::Current(id) => jar
            .add(cookie_util::create("current", id))
            .add(cookie_util::create("last_visited", "none")),

        VTuberId::LastVisited(id) => jar
            .add(cookie_util::create("last_visited", id))
            .add(cookie_util::create("current", "none")),
    }
    .add(cookie_util::create("id", client_data.guest_id));

    Ok((jar, stack))
}

pub async fn touch_grass() -> (StatusCode, Markup) {
    (
        StatusCode::OK,
        html! { "go touch some grass man" }
    )
}
