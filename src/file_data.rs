use std::{
    path::Path,
    time::{SystemTime, UNIX_EPOCH},
};

use axum::http::HeaderMap;
use serde::{Deserialize, Serialize};
use tokio::{
    fs::{read_to_string, File},
    io::AsyncWriteExt,
};

use crate::error::UploadError;

const FILE_DATA_PERMISSION_HEADER_NAME: &str = "File-Data-Download-Permission";
const FILE_DATA_DELETE_AFTER_HEADER_NAME: &str = "File-Data-Delete-After";

fn current_unix_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum FileDataPermission {
    /// It's not possible to download the file
    None,
    /// Any unauthenticated user can download the file independent of the auth configuration
    Unlimited,
}

#[derive(Serialize, Deserialize, Default)]
pub struct FileData {
    #[serde(default)]
    pub download_permission: Option<FileDataPermission>,
    /// delete the file at this timestamp
    #[serde(default)]
    pub ttl: Option<u64>,
}

impl TryFrom<&str> for FileDataPermission {
    type Error = UploadError;
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "none" => Ok(Self::None),
            "unlimited" => Ok(Self::Unlimited),
            _ => Err(UploadError::InvalidFileDataArgument(
                FILE_DATA_PERMISSION_HEADER_NAME.to_string(),
            )),
        }
    }
}

impl TryFrom<HeaderMap> for FileData {
    type Error = UploadError;
    fn try_from(headers: HeaderMap) -> Result<Self, Self::Error> {
        let mut file_data = Self::default();
        if let Some(value) = headers.get(FILE_DATA_PERMISSION_HEADER_NAME) {
            file_data.download_permission = Some(
                value
                    .to_str()
                    .map_err(|_| {
                        UploadError::InvalidFileDataArgument(
                            FILE_DATA_PERMISSION_HEADER_NAME.to_string(),
                        )
                    })?
                    .try_into()?,
            );
        }

        if let Some(value) = headers.get(FILE_DATA_DELETE_AFTER_HEADER_NAME) {
            let string = value.to_str().map_err(|_| {
                UploadError::InvalidFileDataArgument(FILE_DATA_DELETE_AFTER_HEADER_NAME.to_string())
            })?;
            let delete_after = string.parse::<u64>().map_err(|_| {
                UploadError::InvalidFileDataArgument(FILE_DATA_DELETE_AFTER_HEADER_NAME.to_string())
            })?;
            file_data.ttl = Some(current_unix_timestamp() + delete_after);
        }
        Ok(file_data)
    }
}

impl FileData {
    /// this functions returns true if there is no file data (all values are the default ones)
    pub fn is_empty(&self) -> bool {
        self.download_permission.is_none()
    }

    /// checks if the file should be dead
    /// returns `false` if the ttl isn't set
    /// returns `true` if the ttl is in past
    pub fn expired(&self) -> bool {
        if let Some(ttl) = self.ttl {
            if ttl < current_unix_timestamp() {
                return true;
            }
        }
        false
    }

    pub async fn write_to(&self, path: &Path) -> Result<(), UploadError> {
        let serialized = serde_json::to_string(self)?;

        let mut file = File::create(path).await?;
        file.write_all(serialized.as_bytes()).await?;

        Ok(())
    }

    pub async fn read_from(path: &Path) -> Result<Option<Self>, UploadError> {
        if !path.exists() {
            return Ok(None);
        }

        let file_content = read_to_string(path).await?;
        Ok(Some(serde_json::from_str(&file_content)?))
    }
}
