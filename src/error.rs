use std::io;

use axum::{
    body::Body,
    http::{Response, StatusCode},
    response::IntoResponse,
};

macro_rules! error_while_request {
    ($error:expr) => {
        eprintln!(
            "ERROR: There was an error while preparing the response: {:?}",
            $error
        );
    };
}

#[derive(Debug)]
pub enum UploadError {
    InternalServerError,
    /// The user tried to download a file that does not exist
    FileNotExists,
    /// The filename does include invalid characters
    InvalidFilename,
    /// The user tried to upload a file that is bigger than the max_filesize (u64)
    FileIsTooBig(u64),
    /// The requests body is invalid (e.g. empty)
    InvalidBody,
    /// The `Content-Length` header announced more bytes than actually received
    IncompleteUpload(u64, u64),
    /// Invalid authorization includes the required auth scheme
    InvalidAuth(String),
    /// A file data header value is invalid, the string contains the name of the argument
    InvalidFileDataArgument(String),
}

impl From<io::Error> for UploadError {
    fn from(value: io::Error) -> Self {
        error_while_request!(value);
        UploadError::InternalServerError
    }
}

impl From<serde_json::Error> for UploadError {
    fn from(value: serde_json::Error) -> Self {
        error_while_request!(value);
        UploadError::InternalServerError
    }
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
            Self::IncompleteUpload(announced, actually) => (StatusCode::BAD_REQUEST, format!("Incomplete upload. You announced {announced} bytes (content of Content-Length header) but only sent {actually} bytes.")).into_response(),
            Self::InvalidAuth(scheme) => Response::builder()
                .status(StatusCode::UNAUTHORIZED)
                .header("WWW-Authenticate", scheme)
                .body(Body::from("Unauthorized"))
                .unwrap(),
            Self::InvalidFileDataArgument(header) => (StatusCode::BAD_REQUEST, format!("Invalid file data header '{header}'")).into_response()
        }
    }
}
