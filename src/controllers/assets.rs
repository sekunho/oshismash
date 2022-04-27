use axum::{extract::Path, response::IntoResponse, http::HeaderValue};
use hyper::{HeaderMap, StatusCode, header::CONTENT_TYPE};

use std::{path::PathBuf, ffi::OsStr};

pub async fn show(Path(file): Path<String>) -> impl IntoResponse {
    let file_path = PathBuf::from("./public")
        .join(file);
    let mut headers = HeaderMap::new();

    match std::fs::read_to_string(&file_path) {
        Ok(contents) => match file_path.extension().and_then(|str: &OsStr| str.to_str()) {
            Some("css") => {
                headers.insert(CONTENT_TYPE, HeaderValue::from_static("text/css"));
                (StatusCode::OK, headers, contents)
            },
            Some("js") => {
                headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/javascript"));
                (StatusCode::OK, headers, contents)
            },

            _ => (StatusCode::BAD_REQUEST, headers, "400 BAD REQUEST".to_string()),
        },
        Err(_) => (StatusCode::NOT_FOUND, headers, "404 NOT FOUND".to_string())
    }

}
