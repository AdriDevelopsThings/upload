use axum::{
    body::Body,
    http::{Response, StatusCode},
    response::IntoResponse,
};

#[derive(Debug)]
pub enum UploadError {
    InternalServerError,
    FileNotExists,
    InvalidFilename,
    FileIsTooBig(u64),
    InvalidBody,
    InvalidAuth(String),
}

impl IntoResponse for UploadError {
    fn into_response(self) -> axum::response::Response {
        match self {
            Self::InternalServerError => {
                (StatusCode::INTERNAL_SERVER_ERROR, "Internal Server Error").into_response()
            }
            Self::FileNotExists => (StatusCode::NOT_FOUND, "File does not exist").into_response(),
            Self::InvalidFilename => (StatusCode::BAD_REQUEST, "Invalid filename").into_response(),
            Self::FileIsTooBig(max_filesize) =>
                (StatusCode::BAD_REQUEST, format!("The file you tried to upload is too big. The maximum filesize is {max_filesize} bytes.")).into_response(),
            Self::InvalidBody => (StatusCode::BAD_GATEWAY, "Invalid POST body").into_response(),
            Self::InvalidAuth(scheme) => Response::builder()
                .status(StatusCode::UNAUTHORIZED)
                .header("WWW-Authenticate", scheme)
                .body(Body::from("Unauthorized"))
                .unwrap(),
        }
    }
}
