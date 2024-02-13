#[cfg(target_family = "unix")]
use std::os::unix::fs::MetadataExt;
#[cfg(target_family = "windows")]
use std::os::windows::fs::MetadataExt;

use axum::{
    body::Body,
    extract,
    http::{HeaderMap, Response, StatusCode},
};
use tokio::fs::File;
use tokio_util::io::ReaderStream;

use crate::{
    auth::AuthRequest, auth_helper::authorize_by_headers, error::UploadError, state::State,
};

pub async fn download(
    extract::Path(filename): extract::Path<String>,
    extract::State(state): extract::State<State>,
    headers: HeaderMap,
) -> Result<Response<Body>, UploadError> {
    authorize_by_headers(&state, &headers, AuthRequest::Download)?;
    let download_path = state.upload_directory.join(&filename);
    if !download_path.exists() {
        return Err(UploadError::FileNotExists);
    }

    // prepare the request
    let mut response = Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", "application/octet-stream")
        .header(
            "Content-Disposition",
            format!("attachment; filename=\"{filename}\""),
        );

    // if the metadata is valid add the filesize as Content-Length header
    if let Ok(metadata) = download_path.metadata() {
        response = response.header("Content-Length", metadata.size().to_string());
    }

    // open the file and convert it to Body by getting the ReaderStream
    let file = File::open(&download_path).await?;
    let stream = ReaderStream::new(file);
    let body = Body::from_stream(stream);
    Ok(response.body(body).unwrap()) // add the body to the response and finalize it
}
