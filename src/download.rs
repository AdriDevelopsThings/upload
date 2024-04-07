#[cfg(target_family = "unix")]
use std::os::unix::fs::MetadataExt;
#[cfg(target_family = "windows")]
use std::os::windows::fs::MetadataExt;

use axum::{
    body::Body,
    extract,
    http::{HeaderMap, Response, StatusCode},
};
use tokio::fs::{remove_file, File};
use tokio_util::io::ReaderStream;

use crate::{
    auth::AuthRequest,
    auth_helper::authorize_by_headers,
    error::UploadError,
    file_data::{FileData, FileDataPermission},
    state::State,
};

pub async fn download(
    extract::Path(filename): extract::Path<String>,
    extract::State(state): extract::State<State>,
    headers: HeaderMap,
) -> Result<Response<Body>, UploadError> {
    // save the authorize error because downloading the file could be allowed exceptional
    let authorize_error = authorize_by_headers(&state, &headers, AuthRequest::Download).err();

    let download_path = state.upload_directory.join(&filename);
    if !download_path.exists() {
        // respond with the authorize error if the authorization failed to avoid leaking if the file exists
        if let Some(error) = authorize_error {
            return Err(error);
        } else {
            return Err(UploadError::FileNotExists);
        }
    }

    // read file data and parse it
    // we must be sure that the file existence will be leaked at this point (cause of time analysis), even
    // if the user isn't authenticated and the file data doesn't allow unlimited access
    let file_data_path = state.data_directory.join(&filename);
    let file_data = FileData::read_from(&file_data_path)
        .await?
        .unwrap_or_default();
    if let Some(download_permission) = &file_data.download_permission {
        // check if the authorization was unsuccessfull and the download permission doesn't allow unlimited access
        if let Some(authorize_error) = authorize_error {
            if !matches!(download_permission, FileDataPermission::Unlimited) {
                return Err(authorize_error);
            }
        }

        if matches!(download_permission, FileDataPermission::None) {
            // we must be sure that the file existence will be leaked (cause of time analysis) even though
            // the file seems to be not existing
            return Err(UploadError::FileNotExists);
        }
    } else if let Some(authorize_error) = authorize_error {
        // file data doesn't set a download permission but the authorization was not successfull
        return Err(authorize_error);
    }

    if file_data.expired() {
        println!("INFO: File {filename} got removed because the ttl was reached.");
        remove_file(download_path).await?;
        remove_file(file_data_path).await?;
        return Err(UploadError::FileNotExists);
    }

    // user seems to be authorized to download the file by it's header or the file data permissions at this point

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
