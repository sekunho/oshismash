use std::sync::Arc;

use axum::extract::Form;
use axum::response;
use axum::Extension;
use axum_extra::extract::cookie::SameSite;
use axum_extra::extract::cookie::{self, Cookie};
use axum_extra::extract::CookieJar;
use ::cookie::time::Duration;
use maud::Markup;
use serde_json::Value;

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
                guest.guest_id,
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
        .await.map_err(|e| {
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
    Form(form_data): Form<Value>,
    jar: cookie::CookieJar,
) -> response::Result<(CookieJar, Markup), oshismash::Error> {
    // TODO: Move to middleware
    if let Some(cookie) = jar.get("id") {
        let guest_id = cookie.value();
        let client = db_handle.get_client().await?;

        // Check if it's a valid guest ID
        if guests::is_valid(&client, guest_id).await? {
            match form_data {
                Value::Object(mut form_data) => {
                    form_data.insert("guest_id".to_string(), Value::String(guest_id.to_string()));

                    let vote = Vote::from(Value::Object(form_data.clone()))?;
                    let stack = oshismash::vote::vote(&client, vote).await?;

                    let template = views::root::render(
                        "Oshi Smash: Smash or Pass Your Oshis!",
                        views::vote::render(stack.clone()),
                    );

                    let last_voted_cookie = set_last_voted_cookie(stack.clone());
                    let current_cookie = set_current_cookie(stack.clone());
                    let jar = jar.add(last_voted_cookie).add(current_cookie);

                    Ok((jar, template))
                }

                _ => todo!(),
            }
        } else {
            Err(oshismash::Error::InvalidGuest)
        }
    } else {
        Err(oshismash::Error::InvalidGuest)
    }
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

// TODO: Later
// pub async fn rpc_vote(
//     Extension(db_handle): Extension<Arc<db::Handle>>,
//     Json(vote_entry): Json<VoteEntry>,
//     _jar: cookie::CookieJar
// ) -> (StatusCode, Json<Value>) {
//     match db_handle.get_client().await {
//         Ok(client) => {
//             match vtubers::vote(&client, vote_entry).await {
//                 Ok(vtuber) => {
//                     match serde_json::to_value(vtuber) {
//                         Ok(value) => (StatusCode::OK, Json(value)),
//                         Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"bruh": "bruh"})))
//                     }
//                 }
//                 Err(_e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"message": "OH NO"})))
//             }
//         }
//         Err(_e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"message": "OIH NO"})))
//     }
// }
