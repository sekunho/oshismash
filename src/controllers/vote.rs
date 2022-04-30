use std::sync::Arc;

use axum::{Extension, Json};
use axum::extract::Form;
use axum::response::{self, IntoResponse};
use axum_extra::extract::cookie::{self, Cookie};
use hyper::StatusCode;
use maud::{Markup, html};
use serde_json::{Value, json};

use crate::oshismash::vtubers::VoteEntry;
use crate::{views, db};
use crate::oshismash::{self, vtubers, guests};

/// Main page for the smash or pass
pub async fn show(
    Extension(db_handle): Extension<Arc<db::Handle>>,
    jar: cookie::CookieJar
) -> impl IntoResponse {
    // NOTE: Am I supposed to move the cookie stuff to `tower`/middleware?
    // Cookies:
    // https://developer.mozilla.org/en-US/docs/Web/HTTP/Cookies

    let jar = match jar.get("scope").and(jar.get("id")) {
        Some(_) => jar,
        None => {
            // TODO: Refactor cause ugly
            // TODO: REMOVE UNWRAPS
            let client = db_handle.pool.get().await.unwrap();
            let guest = guests::create_guest(&client).await.unwrap();

            let id = format!(
                "id={}; HttpOnly; Secure; SameSite=Strict; Max-Age=2147483647",
                guest.guest_id,
            );

            let scope = "scope=all; HttpOnly; Secure; SameSite=Strict; Max-Age=2147483647";

            let id = Cookie::parse(id).unwrap();
            let scope = Cookie::parse(scope).unwrap();

            jar.add(id).add(scope)
        }
    };

    println!("{:?}", jar);

    (
        jar,
        views::root::render(
            "Oshi Smash: Smash or Pass Your Oshis!",
            views::vote::render()
        )
    )
}

/// Handles the voting for a VTuber
pub async fn vote(
    Extension(db_handle): Extension<Arc<db::Handle>>,
    Form(vote_entry): Form<VoteEntry>,
    jar: cookie::CookieJar,
) -> response::Result<Markup, oshismash::Error> {

    if let Some(cookie) = jar.get("id") {
        println!("{}", cookie.value());

        let foo = cookie.value();

    }

    // let client = db_handle.get_client().await?;
    // let vtuber = vtubers::vote(&client, vote_entry).await?;

    // println!("{:?}", jar);
    // println!("{:?}", vtuber);

    Ok(html! { h1 { ("hey") } })
}

// TODO: Later
pub async fn rpc_vote(
    Extension(db_handle): Extension<Arc<db::Handle>>,
    Json(vote_entry): Json<VoteEntry>,
    _jar: cookie::CookieJar
) -> (StatusCode, Json<Value>) {
    match db_handle.get_client().await {
        Ok(client) => {
            match vtubers::vote(&client, vote_entry).await {
                Ok(vtuber) => {
                    match serde_json::to_value(vtuber) {
                        Ok(value) => (StatusCode::OK, Json(value)),
                        Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"bruh": "bruh"})))
                    }
                }
                Err(_e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"message": "OH NO"})))
            }
        }
        Err(_e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"message": "OIH NO"})))
    }
}
