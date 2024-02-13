use axum::{
    body::{Body, HttpBody},
    extract,
    http::{HeaderMap, Response, StatusCode},
};
use futures_util::StreamExt;
use tokio::{
    fs::{rename, File},
    io::AsyncWriteExt,
};

use crate::{
    auth::{AuthError, AuthRequest},
    error::UploadError,
    random::generate_random_characters,
    state::State,
};

fn validate_filename(filename: &str) -> Result<(), UploadError> {
    if filename.contains('/') {
        return Err(UploadError::InvalidFilename);
    }
    Ok(())
}

pub async fn upload(
    extract::Path(filename): extract::Path<String>,
    extract::State(state): extract::State<State>,
    headers: HeaderMap,
    body: Body,
) -> Result<Response<Body>, UploadError> {
    state
        .auth_config
        .authorize(
            &AuthRequest::Upload,
            headers.get("Authorization").map(|h| h.to_str().unwrap()),
        )
        .map_err(|err| match err {
            AuthError::InvalidAuth(scheme) => match scheme {
                Some(scheme) => UploadError::InvalidAuth(scheme),
                None => UploadError::InvalidAuth(state.auth_config.default_auth_scheme),
            },
        })?;
    validate_filename(&filename)?;
    let filename = generate_random_characters(8) + "_" + &filename;
    let upload_path = state
        .upload_directory
        .clone()
        .join(filename.clone() + ".upload");
    let mut stream = body.into_data_stream();
    if stream.is_end_stream() {
        return Err(UploadError::InvalidBody);
    }
    let mut file = File::create(&upload_path)
        .await
        .map_err(|_| UploadError::InternalServerError)?;
    while let Some(Ok(value)) = stream.next().await {
        file.write_all(&value)
            .await
            .map_err(|_| UploadError::InternalServerError)?;
    }
    rename(&upload_path, state.upload_directory.clone().join(&filename))
        .await
        .map_err(|_| UploadError::InternalServerError)?;
    println!("INFO: Uploaded {filename}");
    Ok(Response::builder()
        .header("Location", format!("/d/{filename}"))
        .header("Content-Type", "text/plain")
        .status(StatusCode::CREATED)
        .body(Body::from(format!("Created: /d/{filename}")))
        .unwrap())
}
