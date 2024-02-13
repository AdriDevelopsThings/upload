use std::{env, path::Path};

use axum::{routing, serve, Router};
use tokio::net::TcpListener;

use crate::{auth::AuthConfig, state::State};

mod auth;
mod auth_helper;
mod download;
mod error;
mod random;
mod state;
mod upload;

#[tokio::main]
async fn main() {
    let auth_config = AuthConfig::read_from_file();
    let upload_directory = env::var("UPLOAD_DIRECTORY").unwrap_or_else(|_| "upload".to_string());
    let state = State::new(auth_config, Path::new(&upload_directory).to_path_buf());

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
