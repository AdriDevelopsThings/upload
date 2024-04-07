use std::{
    fs::{create_dir, read_dir, remove_file},
    path::PathBuf,
};

use crate::auth::AuthConfig;

#[derive(Clone)]
pub struct State {
    pub auth_config: AuthConfig,
    pub upload_directory: PathBuf,
    pub data_directory: PathBuf,
}

impl State {
    pub fn new(
        auth_config: AuthConfig,
        upload_directory: PathBuf,
        data_directory: PathBuf,
    ) -> Self {
        // check if the upload directory exists and create it if not
        if !upload_directory.exists() {
            create_dir(&upload_directory).expect("Error while creating upload directory");
        }

        if !data_directory.exists() {
            create_dir(&data_directory).expect("Error while creating data directory");
        }

        // cleaning up files that wasn't uploaded completely (have a .upload suffix)
        let content =
            read_dir(&upload_directory).expect("Error while listing files of upload directory");
        for file in content {
            let file = file
                .expect("Error while reading dir entry while listing files of upload directory");
            let filename = file.file_name();
            let filename = filename.to_str().expect("Error while transforming file name into string while listing files of upload directory");
            if filename.ends_with(".upload") {
                println!("INFO: Cleaning up file {filename}");
                remove_file(file.path())
                    .expect("Error while removing file that was not uploaded completely");
            }
        }

        // cleaning up data files that doesn't belong to an existing file
        let content =
            read_dir(&data_directory).expect("Error while listing files of data directory");
        for file in content {
            let file =
                file.expect("Error while reading dir etry while listing files of data directory");
            let filename = file.file_name();
            let upload_file_path = upload_directory.join(&filename);
            if !upload_file_path.exists() {
                println!("INFO: Cleaning up data file {}", filename.to_str().expect("Error while transforming file name into string while listing files of data directory"));
                remove_file(file.path()).expect(
                    "Error while removing data file that doesn't have a belonging upload file",
                );
            }
        }

        Self {
            auth_config,
            upload_directory,
            data_directory,
        }
    }
}
