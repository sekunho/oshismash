use std::sync::Arc;

use axum::extract::{Form, FromRequest};
use axum::{async_trait, BoxError, Extension};
use axum_extra::extract::{cookie, CookieJar};
use hyper::header::LOCATION;
use hyper::{HeaderMap, StatusCode};
use maud::{html, Markup};
use serde_json::Value;

use crate::oshismash::guests;
use crate::oshismash::guests::GuestId;
use crate::oshismash::vote::ParseError;
use crate::oshismash::vote::Vote;
use crate::oshismash::vtubers::VTuberId;
use crate::oshismash_web::client_data::ClientData;
use crate::oshismash_web::cookie_util;
use crate::{db, oshismash, config};

#[async_trait]
impl<B> FromRequest<B> for Vote
where
    // Copied these trait bounds from
    // https://docs.rs/axum/latest/axum/extract/struct.Form.html#impl-FromRequest%3CB%3E
    B: Send + axum::body::HttpBody,
    B::Data: Send,
    B::Error: Into<BoxError>,
{
    type Rejection = oshismash::Error;

    async fn from_request(
        req: &mut axum::extract::RequestParts<B>,
    ) -> Result<Self, Self::Rejection> {
        let form_data = req.extract::<Form<Value>>().await;

        let guest_id = req
            .extract::<CookieJar>()
            .await
            .map_err(|_| ParseError::MissingField(String::from("guest_id")))
            .and_then(|jar| {
                jar.get("id")
                    .and_then(|cookie| Some(GuestId(cookie.value().to_string())))
                    .ok_or(ParseError::MissingField(String::from("guest_id")))
            });

        let db = req.extract::<Extension<Arc<db::Handle>>>().await;

        // Hadouken'd :(
        match (form_data, guest_id, db) {
            (Ok(Form(Value::Object(mut form_data))), Ok(GuestId(guest_id)), Ok(db)) => {
                match db.client().await {
                    Ok(client) => match guests::is_valid(&client, guest_id.as_str()).await {
                        Ok(true) => {
                            form_data.insert(String::from("guest_id"), Value::String(guest_id));

                            let result = Vote::from(Value::Object(form_data))?;
                            Ok(result)
                        }
                        Ok(false) => Err(oshismash::Error::InvalidGuest),
                        Err(e) => Err(e),
                    },
                    Err(e) => Err(oshismash::Error::from(e)),
                }
            }
            (Ok(Form(_)), _, _) => Err(oshismash::Error::InvalidFormFormat),
            (_, _, Err(_)) => Err(oshismash::Error::MissingExtension),
            (_, Err(e), _) => Err(oshismash::Error::VoteParseError(e)),
            (Err(e), _, _) => Err(oshismash::Error::InvalidForm(e)),
        }
    }
}

/// Handles the voting for a VTuber
pub async fn vote(
    Extension(db_handle): Extension<Arc<db::Handle>>,
    Extension(app_config): Extension<Arc<config::AppConfig>>,
    client_data: ClientData,
    vote: Vote,
    jar: cookie::CookieJar,
) -> Result<(StatusCode, HeaderMap, CookieJar, Markup), oshismash::Error> {
    // Guest is not allowed to vote if ever the max visited entry is less than
    // the target VTuber ID. But this assumes that the VTubers are in order.
    if client_data.max_visited < vote.vtuber_id {
        Err(oshismash::Error::NotAllowedToVote)
    } else {
        let db_client = db_handle.client().await?;
        let stack = oshismash::vote::vote(&db_client, vote.clone()).await?;

        let vote_list =
            stack
                .get_vote_list()
                .into_iter()
                .fold("".to_string(), |acc, vote| match acc.as_str() {
                    "" => vote.to_string(),
                    acc => format!("{},{}", acc, vote),
                });

        let jar = jar.add(cookie_util::create("voted", vote_list));

        // TODO(sekun): Move to middleware. I think it's possible.
        //
        // This might make no sense, and maybe there's a better way to do this, but
        // the cookies are set this way. If the `current` cookie is set with an
        // actual value, that is the VTuber's ID, then `last_visited` should be set
        // to `none` because there's literally no use for it. The only time it is
        // ever used is when `current` is `none`.
        match stack.get_current() {
            Some(vtuber) => {
                let jar = jar
                    .add(cookie_util::create("current", vtuber.id))
                    .add(cookie_util::create("last_visited", "none"));

                let jar = if vtuber.id > client_data.max_visited {
                    jar.add(cookie_util::create("max_visited", vtuber.id))
                } else {
                    jar
                };

                let mut headers = HeaderMap::new();
                // TODO: Refactor this
                let url = format!("{}/{}", app_config.base_url(), vtuber.id);

                headers.insert(LOCATION, url.parse().unwrap());

                Ok((StatusCode::FOUND, headers, jar, html! {}))
            }
            None => match client_data.vtuber_id {
                VTuberId::Current(id) => {
                    let jar = jar
                        .add(cookie_util::create("last_visited", id))
                        .add(cookie_util::create("current", "none"));

                    let mut headers = HeaderMap::new();

                    // TODO: Refactor this
                    headers.insert(LOCATION, app_config.base_url().parse().unwrap());

                    Ok((StatusCode::FOUND, headers, jar, html! {}))
                }
                VTuberId::LastVisited(_) => Err(oshismash::Error::InvalidClientData),
            },
        }
    }
}
