use std::{fs::create_dir, path::PathBuf};

use crate::auth::AuthConfig;

#[derive(Clone)]
pub struct State {
    pub auth_config: AuthConfig,
    pub upload_directory: PathBuf,
}

impl State {
    pub fn new(auth_config: AuthConfig, upload_directory: PathBuf) -> Self {
        if !upload_directory.exists() {
            create_dir(&upload_directory).expect("Error while creating upload directory");
        }
        Self {
            auth_config,
            upload_directory,
        }
    }
}
