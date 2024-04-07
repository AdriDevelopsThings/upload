use std::{env, path::PathBuf};

use axum::{routing, serve, Router};
use tokio::net::TcpListener;

use crate::{auth::AuthConfig, state::State};

mod auth;
mod auth_helper;
mod download;
mod error;
mod file_data;
mod random;
mod state;
mod ttl_killer;
mod upload;

#[tokio::main]
async fn main() {
    let auth_config = AuthConfig::read_from_file();
    let upload_directory = env::var("UPLOAD_DIRECTORY").unwrap_or_else(|_| "upload".to_string());
    let data_directory = env::var("DATA_DIRECTORY").unwrap_or_else(|_| "data".to_string());
    // check upload and data directory values
    if upload_directory == data_directory
        || PathBuf::from(&data_directory).starts_with(&upload_directory)
    {
        panic!("Data directory cannot be the same directory as the upload directory or be a subdirectory of it.\nChange the 'UPLOAD_DIRECTORY' or 'DATA_DIRECTORY' environment variable to another one.");
    }
    let state = State::new(
        auth_config,
        PathBuf::from(&upload_directory),
        PathBuf::from(&data_directory),
    );

    ttl_killer::start_ttl_killer(state.clone());

    let router = Router::new()
        .route("/upload/:filename", routing::post(upload::upload))
        .route("/d/:filename", routing::get(download::download))
        .with_state(state);

    let listen_address =
        env::var("LISTEN_ADDRESS").unwrap_or_else(|_| "127.0.0.1:3000".to_string());
    let listener = TcpListener::bind(&listen_address)
        .await
        .expect("Error while listening on listen address");
    println!("INFO: Server listening on http://{listen_address}");
    serve(listener, router)
        .await
        .expect("Error while serving http server");
}
