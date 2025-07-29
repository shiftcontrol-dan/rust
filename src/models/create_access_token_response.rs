use std::convert::TryFrom;
use schemars::JsonSchema;

#[derive(Clone, Debug, PartialEq, Default, Serialize, Deserialize, JsonSchema)]
pub struct CreateAccessTokenResponse {
    pub access_token: String,
}

impl CreateAccessTokenResponse {
    pub fn new(access_token: String) -> Self {
        CreateAccessTokenResponse {
            access_token,
        }
    }
}
