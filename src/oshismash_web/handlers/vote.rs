use std::sync::Arc;

use axum::Extension;
use axum_extra::extract::cookie;
use axum_extra::extract::CookieJar;

use crate::db;
use crate::oshismash;
use crate::oshismash::vote::Vote;
use crate::oshismash::vtubers::Stack;
use crate::oshismash::vtubers::VTuberId;
use crate::oshismash_web::client_data::ClientData;
use crate::oshismash_web::cookie_util;

/// Handles the voting for a VTuber
pub async fn vote(
    Extension(db_handle): Extension<Arc<db::Handle>>,
    client_data: ClientData,
    vote: Vote,
    jar: cookie::CookieJar,
) -> Result<(CookieJar, Stack), oshismash::Error> {
    // Guest is not allowed to vote if ever the max visited entry is less than
    // the target VTuber ID. But this assumes that the VTubers are in order.
    if client_data.max_visited < vote.vtuber_id {
        Err(oshismash::Error::NotAllowedToVote)
    } else {
        let db_client = db_handle.get_client().await?;
        let stack = oshismash::vote::vote(&db_client, vote.clone()).await?;

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

                Ok((jar, stack))
            }
            None => match client_data.vtuber_id {
                VTuberId::Current(id) => {
                    let jar = jar
                        .add(cookie_util::create("last_visited", id))
                        .add(cookie_util::create("current", "none"));

                    Ok((jar, stack))
                }
                VTuberId::LastVisited(_) => Err(oshismash::Error::InvalidClientData),
            },
        }
    }
}
