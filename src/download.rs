use std::os::unix::fs::MetadataExt;

use axum::{
    body::Body,
    extract,
    http::{Response, StatusCode},
};
use tokio::fs::File;
use tokio_util::io::ReaderStream;

use crate::{error::UploadError, state::State};

pub async fn download(
    extract::Path(filename): extract::Path<String>,
    extract::State(state): extract::State<State>,
) -> Result<Response<Body>, UploadError> {
    let download_path = state.upload_directory.join(&filename);
    if !download_path.exists() {
        return Err(UploadError::FileNotExists);
    }
    let mut response = Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", "application/octet-stream")
        .header(
            "Content-Disposition",
            format!("attachment; filename=\"{filename}\""),
        );
    if let Ok(metadata) = download_path.metadata() {
        response = response.header("Content-Size", metadata.size().to_string());
    }
    let file = File::open(&download_path)
        .await
        .map_err(|_| UploadError::InternalServerError)?;
    let stream = ReaderStream::new(file);
    let body = Body::from_stream(stream);
    Ok(response.body(body).unwrap())
}
