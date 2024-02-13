use bcrypt::verify;
use serde::Deserialize;

use super::{AuthError, AuthRequest};

fn true_fn() -> bool {
    true
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

pub struct BasicAuthArgument<'a> {
    pub username: &'a str,
    pub password: &'a str,
}

impl BasicAuth {
    pub fn authorize(
        &self,
        request: &AuthRequest,
        argument: BasicAuthArgument,
    ) -> Result<Option<u64>, AuthError> {
        if self.username == argument.username {
            // if the password is correct and this authorization does allow the required `AuthRequest`
            if verify(argument.password, &self.password).unwrap_or_default()
                && ((request == &AuthRequest::Download && self.allow_download)
                    || (request == &AuthRequest::Upload && self.allow_upload))
            {
                return Ok(self.max_filesize);
            }
            // incorrect password or permission, but the auth scheme seems to be right
            return Err(AuthError::InvalidAuth(Some("Basic".to_string())));
        }
        // the username does not match
        Err(AuthError::InvalidAuth(None))
    }
}
