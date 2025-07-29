use std::convert::TryFrom;
use schemars::JsonSchema;

#[derive(Clone, Debug, PartialEq, Default, Serialize, Deserialize, JsonSchema)]
pub struct CreateSamlConnectionLinkResponse {
    #[serde(rename = "url")]
    pub url: String,
}

impl CreateSamlConnectionLinkResponse {
    pub fn new(url: String) -> CreateSamlConnectionLinkResponse {
        CreateSamlConnectionLinkResponse { url }
    }
}
