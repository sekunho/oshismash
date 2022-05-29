use axum::response::{IntoResponse, Response};

use super::views;
use crate::oshismash::vtubers::Stack;

impl IntoResponse for Stack {
    fn into_response(self) -> Response {
        views::root::render(
            "Oshi Smash: Smash or Pass Your Oshis!",
            views::vote::render(self),
        )
        .into_response()
    }
}
