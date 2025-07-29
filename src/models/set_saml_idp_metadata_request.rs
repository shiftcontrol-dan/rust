use std::convert::TryFrom;
use schemars::JsonSchema;

#[derive(Clone, Debug, PartialEq, Serialize, JsonSchema)]
pub struct SetSamlIdpMetadataRequest {
    #[serde(rename = "org_id")]
    pub org_id: String,
    #[serde(rename = "idp_entity_id")]
    pub idp_entity_id: String,
    #[serde(rename = "idp_sso_url")]
    pub idp_sso_url: String,
    #[serde(rename = "idp_certificate")]
    pub idp_certificate: String,
    #[serde(rename = "provider")]
    pub provider: SamlIdpProvider,
}

#[derive(Clone, Debug, PartialEq, Serialize, JsonSchema)]
pub enum SamlIdpProvider {
    Google,
    Rippling,
    OneLogin,
    JumpCloud,
    Okta,
    Azure,
    Duo,
    Generic,
}
