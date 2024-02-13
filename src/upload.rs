use axum::{
    body::{Body, HttpBody},
    extract,
    http::{HeaderMap, Response, StatusCode},
};
use blake3::Hasher;
use futures_util::StreamExt;
use tokio::{
    fs::{remove_file, rename, File},
    io::AsyncWriteExt,
};

use crate::{
    auth::AuthRequest, auth_helper::authorize_by_headers, error::UploadError,
    random::generate_random_characters, state::State,
};

/// checks if filename is invalid and return `Err(UploadError::InvalidFilename)` if not
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
    // authorize the request by its `Authorization` header
    let max_filesize = authorize_by_headers(&state, &headers, AuthRequest::Upload)?;

    let mut content_size: Option<u64> = None;
    // check if the user tries to upload a file that is too big or empty if he gave us the `Content-Length` header
    if let Some(content_size_header) = headers.get("Content-Length") {
        if let Ok(Ok(content_size_from_header)) =
            content_size_header.to_str().map(|s| s.parse::<u64>())
        {
            if content_size_from_header == 0 {
                return Err(UploadError::InvalidBody);
            }
            if content_size_from_header > max_filesize {
                return Err(UploadError::FileIsTooBig(max_filesize));
            }
            content_size = Some(content_size_from_header);
        }
    }

    validate_filename(&filename)?;

    // the filename starts with random characters
    let upload_filename = generate_random_characters(8) + "_" + &filename + ".upload";

    // the file will have a `.upload` suffix until the upload is finished
    let upload_path = state.upload_directory.clone().join(&upload_filename);
    let mut stream = body.into_data_stream();

    // if the body seems to be empty return a invalid body error
    if stream.is_end_stream() {
        return Err(UploadError::InvalidBody);
    }

    let mut file = File::create(&upload_path).await?;

    // size will contain the already uploaded filesize
    let mut size: u64 = 0;
    // start hashing the file by creating a blake3 hasher
    let mut hasher = Hasher::new();
    while let Some(Ok(value)) = stream.next().await {
        size += value.len() as u64;
        // the file got to big, remove the file and return a file is too big error
        if size > max_filesize {
            // we will close the file before removing because you can't remove the file before closing the file on windows
            drop(file);
            remove_file(&upload_path).await?;
            return Err(UploadError::FileIsTooBig(max_filesize));
        }
        // write the chunk to the file
        file.write_all(&value).await?;
        hasher.update(&value);
    }
    drop(file);

    // check if the upload was completed
    if let Some(content_size) = content_size {
        if content_size > size {
            println!("ERROR: There was an error while uploading {upload_filename}. The upload seems to be incomplete: The user tried to upload {content_size} bytes but only {size} bytes was received.");
            remove_file(&upload_path).await?;
            return Err(UploadError::IncompleteUpload(content_size, size));
        }
    }
    // compute the blake3 hash
    let blake3_hex = hasher.finalize().to_hex().to_string();
    // the real filename has the format {first 8 characters of blake3 hash (hex) of file}_{filename}
    let real_filename = format!("{}_{filename}", &blake3_hex[..8]);
    let real_path = state.upload_directory.clone().join(&real_filename);

    // the upload is completed so the file will be renamed to the correct filename
    rename(&upload_path, real_path).await?;
    println!("INFO: Uploaded {real_filename}");

    // respond with a CREATED response that includes the link to the created file in body and the `Location` header
    Ok(Response::builder()
        .header("Location", format!("/d/{real_filename}"))
        .header("Content-Type", "text/plain")
        .status(StatusCode::CREATED)
        .body(Body::from(format!("Created: /d/{real_filename}")))
        .unwrap())
}
