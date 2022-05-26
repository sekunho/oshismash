use std::sync::Arc;

use ::cookie::time::Duration;
use axum::Extension;
use axum_extra::extract::cookie::SameSite;
use axum_extra::extract::cookie::{self, Cookie};
use axum_extra::extract::CookieJar;
use maud::Markup;

use crate::oshismash::vote::Vote;
use crate::oshismash::vtubers::Stack;
use crate::oshismash::{self, guests, vtubers};
use crate::{db, views};

/// Main page for the smash or pass
pub async fn index(
    Extension(db_handle): Extension<Arc<db::Handle>>,
    jar: cookie::CookieJar,
) -> Result<(cookie::CookieJar, Markup), oshismash::Error> {
    // NOTE: Am I supposed to move the cookie stuff to `tower`/middleware?
    // Cookies:
    // https://developer.mozilla.org/en-US/docs/Web/HTTP/Cookies
    let jar = match jar.get("scope").and(jar.get("id")) {
        Some(_) => jar,
        None => {
            // TODO: Refactor cause ugly
            // TODO: REMOVE UNWRAPS
            let client = db_handle.pool.get().await?;
            let guest = guests::create_guest(&client).await.unwrap();

            let id = format!(
                "id={}; HttpOnly; Secure; SameSite=Strict; Max-Age=2147483647",
                guest.guest_id.0,
            );

            let scope = "scope=all; HttpOnly; Secure; SameSite=Strict; Max-Age=2147483647";

            let id = Cookie::parse(id).unwrap();
            let scope = Cookie::parse(scope).unwrap();

            jar.add(id).add(scope)
        }
    };

    let client = db_handle.pool.get().await?;

    let last_visited_id = jar
        .get("last_visited")
        .and_then(|c| c.value().parse::<i64>().ok());

    let stack = vtubers::get_vote_stack(&client, last_visited_id)
        .await
        .map_err(|e| {
            println!("{:?}", e);
            e
        })?;

    let template = (
        jar,
        views::root::render(
            "Oshi Smash: Smash or Pass Your Oshis!",
            views::vote::render(stack),
        ),
    );

    Ok(template)
}

/// Handles the voting for a VTuber
pub async fn vote(
    Extension(db_handle): Extension<Arc<db::Handle>>,
    vote: Vote,
    jar: cookie::CookieJar,
) -> Result<(CookieJar, Markup), oshismash::Error> {
    let client = db_handle.get_client().await?;
    let stack = oshismash::vote::vote(&client, vote).await?;

    // TODO(sekun): Move to an `IntoResponse` instance
    let template = views::root::render(
        "Oshi Smash: Smash or Pass Your Oshis!",
        views::vote::render(stack.clone()),
    );

    // TODO(sekun): Move to middleware. I think it's possible.
    let last_voted_cookie = set_last_voted_cookie(stack.clone());
    let current_cookie = set_current_cookie(stack.clone());
    let jar = jar.add(last_voted_cookie).add(current_cookie);

    Ok((jar, template))
}

fn set_current_cookie<'a>(stack: Stack) -> Cookie<'a> {
    let cookie_val = match stack.get_current() {
        Some(current) => current.id.to_string(),
        None => "none".to_string(),
    };

    Cookie::new("current", cookie_val)
}

fn set_last_voted_cookie<'a>(stack: Stack) -> Cookie<'a> {
    let cookie_val = match stack.get_last_voted_stat() {
        Some(stat) => stat.vtuber_id.to_string(),
        None => "none".to_string(),
    };

    let mut cookie = Cookie::new("last_visited", cookie_val);

    cookie.set_secure(true);
    cookie.set_http_only(true);
    cookie.set_same_site(SameSite::Strict);
    cookie.set_max_age(Duration::seconds(2147483647));

    cookie
}
