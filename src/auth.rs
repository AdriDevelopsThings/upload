use std::{env, fs::read_to_string};

use base64::prelude::*;
use bcrypt::verify;
use serde::Deserialize;

fn true_fn() -> bool {
    true
}

fn default_auth_scheme() -> String {
    "Basic".to_string()
}

fn default_max_filesize() -> u64 {
    1024 * 1024 * 1024 * 10 // 10 GB
}

#[derive(PartialEq)]
pub enum AuthRequest {
    Upload,
    Download,
}

#[derive(Debug)]
pub enum AuthError {
    InvalidAuth(Option<String>),
}

#[derive(Clone, Deserialize)]
pub struct AuthConfig {
    #[serde(default = "default_auth_scheme")]
    pub default_auth_scheme: String,
    #[serde(default = "default_max_filesize")]
    pub default_max_filesize: u64,
    #[serde(default)]
    pub allow_uploading_for_everyone: bool,
    #[serde(default)]
    pub allow_downloading_for_everyone: bool,
    #[serde(default)]
    pub basic: Vec<BasicAuth>,
}

#[derive(Clone, Deserialize)]
pub struct BasicAuth {
    pub username: String,
    pub password: String,
    #[serde(default)]
    pub max_filesize: Option<u64>,
    #[serde(default = "true_fn")]
    pub allow_download: bool,
    #[serde(default = "true_fn")]
    pub allow_upload: bool,
}

impl AuthConfig {
    pub fn read_from_file() -> Self {
        let path = env::var("AUTH_CONFIG_PATH").unwrap_or_else(|_| "auth.toml".to_string());
        let toml_content = read_to_string(path).expect("Error while reading auth config file");
        let config: AuthConfig =
            toml::from_str(&toml_content).expect("Error while parsing auth config file");
        if config.basic.is_empty() {
            eprintln!(
                "WARNING: You didn't configure any auth scheme. Uploading will be impossible."
            );
        }
        config
    }

    pub fn authorize(
        &self,
        request: &AuthRequest,
        authorization: Option<&str>,
    ) -> Result<u64, AuthError> {
        if (request == &AuthRequest::Download && self.allow_downloading_for_everyone)
            || (request == &AuthRequest::Upload && self.allow_uploading_for_everyone)
        {
            return Ok(self.default_max_filesize);
        }
        if authorization.is_none() {
            return Err(AuthError::InvalidAuth(None));
        }
        let authorization = authorization.unwrap();
        let splitted = authorization.split(' ').collect::<Vec<&str>>();
        if splitted.len() != 2 {
            return Err(AuthError::InvalidAuth(None));
        }
        if splitted[0] == "Basic" {
            let decoded = String::from_utf8(
                BASE64_STANDARD
                    .decode(splitted[1])
                    .map_err(|_| AuthError::InvalidAuth(None))?,
            )
            .map_err(|_| AuthError::InvalidAuth(None))?;
            let decoded = decoded.split(':').collect::<Vec<&str>>();
            if decoded.len() != 2 {
                return Err(AuthError::InvalidAuth(None));
            }
            for auth in &self.basic {
                let auth_resp = auth.authorize(request, decoded[0], decoded[1]);
                if let Err(e) = auth_resp {
                    if let AuthError::InvalidAuth(Some(_)) = &e {
                        return Err(e);
                    }
                } else if let Ok(filesize) = auth_resp {
                    return Ok(filesize.unwrap_or_else(|| self.default_max_filesize));
                }
            }
        }
        Err(AuthError::InvalidAuth(None))
    }
}

impl BasicAuth {
    fn authorize(
        &self,
        request: &AuthRequest,
        username: &str,
        password: &str,
    ) -> Result<Option<u64>, AuthError> {
        if self.username == username {
            if verify(password, &self.password).unwrap_or_default()
                && ((request == &AuthRequest::Download && self.allow_download)
                    || (request == &AuthRequest::Upload && self.allow_upload))
            {
                return Ok(self.max_filesize);
            }
            return Err(AuthError::InvalidAuth(Some("Basic".to_string())));
        }
        Err(AuthError::InvalidAuth(None))
    }
}
