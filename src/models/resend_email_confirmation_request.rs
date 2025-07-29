use std::convert::TryFrom;
use schemars::JsonSchema;

#[derive(Clone, Debug, PartialEq, Default, Serialize, Deserialize, JsonSchema)]
pub struct ResendEmailConfirmationRequest {
    #[serde(rename = "user_id")]
    pub user_id: String,
}

impl ResendEmailConfirmationRequest {
    pub fn new(user_id: String) -> ResendEmailConfirmationRequest {
        ResendEmailConfirmationRequest { user_id }
    }
}
