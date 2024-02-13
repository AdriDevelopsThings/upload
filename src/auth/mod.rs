use std::{env, fs::read_to_string};

use crate::auth::{
    basic::{BasicAuth, BasicAuthArgument},
    bearer::BearerAuthArgument,
};
use base64::prelude::*;
use serde::Deserialize;

use self::bearer::BearerAuthConfig;

mod basic;
mod bearer;

/// iterate through a auth method until the authorization was successful or
/// the method responded with a InvalidAuth error that contains a scheme (the authorization was partially successful)
macro_rules! iterate_auth_method {
    ($method:expr, $default_max_filesize:expr, $request:expr, $argument:expr) => {
        for auth in $method {
            let auth_resp = auth.authorize($request, $argument);
            if let Ok(filesize) = auth_resp {
                return Ok(filesize.unwrap_or($default_max_filesize));
            } else if let Err(e) = auth_resp {
                if let AuthError::InvalidAuth(Some(_)) = &e {
                    return Err(e);
                }
            }
        }
    };
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
    /// The authorization was invalid and may include the failed auth scheme
    InvalidAuth(Option<String>),
}

#[derive(Clone, Deserialize)]
pub struct AuthConfig {
    /// This value will be responded in the WWW-Authenticate header by default
    #[serde(default = "default_auth_scheme")]
    pub default_auth_scheme: String,
    /// The maximal filesize if the auth method does not provide another one
    #[serde(default = "default_max_filesize")]
    pub default_max_filesize: u64,
    #[serde(default)]
    pub allow_downloading_for_everyone: bool,
    #[serde(default)]
    pub allow_uploading_for_everyone: bool,
    #[serde(default)]
    pub basic: Vec<BasicAuth>,
    #[serde(default)]
    pub bearer: Vec<BearerAuthConfig>,
}

impl AuthConfig {
    pub fn read_from_file() -> Self {
        let path = env::var("AUTH_CONFIG_PATH").unwrap_or_else(|_| "auth.toml".to_string());
        let toml_content = read_to_string(path).expect("Error while reading auth config file");
        let config: AuthConfig =
            toml::from_str(&toml_content).expect("Error while parsing auth config file");
        if (!config.allow_downloading_for_everyone || !config.allow_uploading_for_everyone)
            && config.basic.is_empty()
        {
            eprintln!(
                "WARNING: You didn't configure any auth scheme. Downloading / uploading will be impossible."
            );
        }
        config
    }

    pub fn authorize(
        &self,
        request: &AuthRequest,
        authorization: Option<&str>,
    ) -> Result<u64, AuthError> {
        // check if no authorization is required
        if (request == &AuthRequest::Download && self.allow_downloading_for_everyone)
            || (request == &AuthRequest::Upload && self.allow_uploading_for_everyone)
        {
            return Ok(self.default_max_filesize);
        }

        // authorization is required
        if authorization.is_none() {
            return Err(AuthError::InvalidAuth(None));
        }
        let authorization = authorization.unwrap();

        // the authorization string should look like this: `<auth scheme> <payload>`
        let splitted = authorization.split(' ').collect::<Vec<&str>>();
        if splitted.len() != 2 {
            return Err(AuthError::InvalidAuth(None));
        }

        if splitted[0] == "Basic" {
            // decode the payload with base64
            let decoded = String::from_utf8(
                BASE64_STANDARD
                    .decode(splitted[1])
                    .map_err(|_| AuthError::InvalidAuth(None))?,
            )
            .map_err(|_| AuthError::InvalidAuth(None))?;

            // the decoded payload looks like this: `username:password`
            let decoded = decoded.split(':').collect::<Vec<&str>>();
            if decoded.len() != 2 {
                return Err(AuthError::InvalidAuth(None));
            }
            iterate_auth_method!(
                &self.basic,
                self.default_max_filesize,
                request,
                BasicAuthArgument {
                    username: decoded[0],
                    password: decoded[1],
                }
            );
        } else if splitted[0] == "Bearer" {
            iterate_auth_method!(
                &self.bearer,
                self.default_max_filesize,
                request,
                BearerAuthArgument(splitted[1].to_string())
            );
        }
        Err(AuthError::InvalidAuth(None))
    }
}
