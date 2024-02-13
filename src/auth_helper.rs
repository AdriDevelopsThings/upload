use axum::http::HeaderMap;

use crate::{
    auth::{AuthError, AuthRequest},
    error::UploadError,
    state::State,
};

pub fn authorize_by_headers(
    state: &State,
    headers: &HeaderMap,
    request: AuthRequest,
) -> Result<u64, UploadError> {
    state
        .auth_config
        .authorize(
            &request,
            headers.get("Authorization").map(|h| h.to_str().unwrap()),
        )
        .map_err(|err| match err {
            AuthError::InvalidAuth(scheme) => match scheme {
                Some(scheme) => UploadError::InvalidAuth(scheme), // the authorization was partially correct, we do know the correct authorization scheme
                None => UploadError::InvalidAuth(state.auth_config.default_auth_scheme.clone()), // we take the default authorization scheme
            },
        })
}
