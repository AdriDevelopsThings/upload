use jsonwebtoken::{decode, DecodingKey, Validation};
use serde::Deserialize;
use serde_json::Value;

use super::{AuthError, AuthRequest};

/// just returns a invalid auth error that contains 'Bearer' as scheme
macro_rules! invalid_auth_bearer {
    () => {
        AuthError::InvalidAuth(Some("Bearer".to_string()))
    };
}

fn default_permissions() -> Vec<String> {
    vec!["download".to_string(), "upload".to_string()]
}

fn default_max_filesize_field_name() -> String {
    "max_filesize".to_string()
}

fn default_permissions_field_name() -> String {
    "permissions".to_string()
}

#[derive(Clone, Deserialize)]
pub struct BearerAuthConfig {
    pub secret: String,
    pub default_max_filesize: Option<u64>,
    #[serde(default = "default_permissions")] // ["download", "upload"]
    pub default_permissions: Vec<String>,
    #[serde(default = "default_max_filesize_field_name")] // "max_filesize"
    pub max_filesize_field_name: String,
    #[serde(default = "default_permissions_field_name")] // "permissions"
    pub permissions_field_name: String,
}

pub struct BearerAuthArgument(pub String);

impl BearerAuthConfig {
    fn get_decoding_key(&self) -> DecodingKey {
        DecodingKey::from_secret(self.secret.as_ref())
    }

    pub fn authorize(
        &self,
        request: &AuthRequest,
        BearerAuthArgument(token): BearerAuthArgument,
    ) -> Result<Option<u64>, AuthError> {
        // decode the jsonwebtoken and extract the claims as `serde_json::Value`
        let token = decode::<Value>(&token, &self.get_decoding_key(), &Validation::default())
            .map_err(|_| AuthError::InvalidAuth(None))?;
        // claim must be a `Value::Object`
        if let Value::Object(map) = token.claims {
            // max_filesize is claim.`configured max_filesize field name` or `configured default_max_filesize`
            let max_filesize = match map
                .get(&self.max_filesize_field_name)
                .map(|value| value.as_number().map(|number| number.as_u64()))
            {
                Some(Some(value)) => value,
                _ => self.default_max_filesize,
            };

            // permissions are claim.`configured permissions field name` or `configured default_permissions`
            let permissions = match map
                .get(&self.permissions_field_name)
                .map(|value| {
                    value.as_array().map(|array| {
                        array
                            .iter()
                            .filter_map(|element| element.as_str().map(|v| v.to_string()))
                            .collect::<Vec<String>>()
                    })
                })
                .map(|v| v.ok_or(invalid_auth_bearer!()))
            {
                Some(Ok(value)) => value,
                _ => self.default_permissions.clone(),
            };

            // check if the permissions include the right request ('download' or 'upload')
            if (request == &AuthRequest::Download && !permissions.contains(&"download".to_string()))
                || (request == &AuthRequest::Upload && !permissions.contains(&"upload".to_string()))
            {
                return Err(invalid_auth_bearer!());
            }

            return Ok(max_filesize);
        }
        Err(AuthError::InvalidAuth(None))
    }
}
