use std::sync::Arc;

use axum::{extract::{FromRequest, Path}, async_trait, Extension};
use axum_extra::extract::CookieJar;

use crate::{oshismash::{self, guests, vtubers::VTuberId}, db};

/// Contains the settings and other data from the client-side of things.
#[derive(Debug)]
pub struct ClientData {
    pub guest_id: String,
    pub vtuber_id: VTuberId,
}

/// Represents the VTuber in the client's UI. This information is stored in 2
/// cookies: `last_visited` and `current`. If `current` is `"none"`, then this
/// should be encoded as `LastVisited`, otherwise it's `Current`.
/// The reason why it's only one or the other is there won't ever be a time that
/// both need to be present simultaneously.
pub enum VTuberIdError {
    Missing,
}

impl TryFrom<&CookieJar> for VTuberId {
    type Error = VTuberIdError;

    fn try_from(jar: &CookieJar) -> Result<Self, Self::Error> {
        let current_id = jar
            .get("current")
            .and_then(|c| c.value().to_string().parse::<i64>().ok());

        match current_id {
            Some(id) => Ok(VTuberId::Current(id)),
            None => {
                let last_visited_id = jar
                    .get("last_visited")
                    .and_then(|c| {
                        c
                            .value()
                            .to_string()
                            .parse::<i64>()
                            .ok()
                    });

                match last_visited_id {
                    Some(id) => Ok(VTuberId::LastVisited(id)),
                    None => Err(VTuberIdError::Missing),
                }
            },
        }
    }
}

#[async_trait]
impl<B> FromRequest<B> for ClientData
where
    B: Send
{
    type Rejection = oshismash::Error;

    async fn from_request(
        req: &mut axum::extract::RequestParts<B>,
    ) -> Result<Self, Self::Rejection> {
        // NOTE: It's infallible so I guess it's safe to unwrap?
        let jar = req.extract::<CookieJar>().await.unwrap();
        let db = req.extract::<Extension<Arc<db::Handle>>>().await?;
        let vtuber_id_path = req
            .extract::<Path<String>>()
            .await;

        let vtuber_id =
            match vtuber_id_path {
                Ok(Path(path)) => {
                    match path.parse::<i64>() {
                        Ok(id) => VTuberId::Current(id),
                        Err(_) => VTuberId::Current(1),
                    }
                },
                Err(_) => {
                    match VTuberId::try_from(&jar) {
                        Ok(vtuber_id) => vtuber_id,
                        Err(_) => VTuberId::Current(1),
                    }
                },
            };

        let guest_id = jar
            .get("id")
            .and_then(|c| Some(c.value().to_string()));

        match guest_id {
            Some(guest_id) => Ok(ClientData {vtuber_id, guest_id}),
            None => {
                let client = db.pool.get().await?;

                guests::create_guest(&client)
                    .await
                    .and_then(|g| {
                        Ok(ClientData {vtuber_id, guest_id: g.guest_id.0})
                    })
            },
        }
    }
}
