use std::sync::Arc;

use axum::Extension;
use axum_extra::extract::cookie;
use hyper::{header::LOCATION, HeaderMap, StatusCode};
use maud::{html, Markup};

use crate::{db, config};
use crate::oshismash_web::client_data::ClientData;
use crate::oshismash_web::views;
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
    Extension(db_handle): Extension<Arc<db::Handle>>,
    Extension(app_config): Extension<Arc<config::AppConfig>>
) -> Result<(StatusCode, HeaderMap, cookie::CookieJar, Markup), oshismash::Error> {
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
    .add(cookie_util::create("id", client_data.guest_id.clone()));

    let mut headers = HeaderMap::new();

    match client_data.vtuber_id {
        VTuberId::Current(id) => {
            // TODO: Use app config to generate URL.
            let url = format!("{}/{}", app_config.base_url(), id);
            headers.insert(LOCATION, url.parse().unwrap());
            Ok((StatusCode::FOUND, headers, jar, html! {}))
        }
        VTuberId::LastVisited(_) => {
            let client = db_handle.pool.get().await?;

            let stack = vtubers::get_vote_stack(
                &client,
                &client_data.vtuber_id,
                client_data.guest_id.clone(),
            )
            .await?;

            let render = (
                StatusCode::OK,
                headers,
                jar,
                views::root::render(
                    "Oshi Smash: Smash or Pass Your Oshis!",
                    views::vote::render(stack),
                ),
            );

            Ok(render)
        }
    }
}

pub async fn show_given_id(
    Extension(db_handle): Extension<Arc<db::Handle>>,
    client_data: ClientData,
    jar: cookie::CookieJar,
) -> Result<(cookie::CookieJar, Stack), oshismash::Error> {
    println!("From ID: {:?}", client_data);
    let client = db_handle.pool.get().await?;

    let stack = vtubers::get_vote_stack(
        &client,
        &client_data.vtuber_id,
        client_data.guest_id.clone(),
    )
    .await
    .map_err(|e| {
        println!("{:?}", e);
        e
    })?;

    println!("{:?}", stack);

    let mut vote_list = stack.get_vote_list();

    if let Some(_id) = client_data.vtuber_id.get_current() {
        vote_list.dedup();
    }

    println!("{:?}", vote_list);

    let visited_list = vote_list
        .into_iter()
        .fold("".to_string(), |acc, vote| match acc.as_str() {
            "" => vote.to_string(),
            acc => {
                format!("{},{}", acc, vote)
            }
        });

    println!("{}", visited_list);

    let jar = match client_data.vtuber_id {
        VTuberId::Current(id) => {
            if client_data.max_visited < id {
                // TODO: Redirect with flash message
                Err(oshismash::Error::MaxVisitedIsLessThanCurrent)
            } else {
                let jar = jar
                    .add(cookie_util::create("current", id))
                    .add(cookie_util::create("last_visited", "none"));

                let jar = if let None = jar.get("max_visited") {
                    jar.add(cookie_util::create("max_visited", client_data.max_visited))
                } else {
                    jar
                };

                Ok(jar)
            }
        }

        VTuberId::LastVisited(id) => {
            let jar = jar
                .add(cookie_util::create("last_visited", id))
                .add(cookie_util::create("current", "none"));

            Ok(jar)
        }
    }?
    .add(cookie_util::create("id", client_data.guest_id))
    .add(cookie_util::create("voted", visited_list));

    Ok((jar, stack))
}

pub async fn details() -> Result<(), oshismash::Error> {
    Ok(())
}
