use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone, schemars::JsonSchema, tsify::Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
pub struct GetProfileQuery {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub slug: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, schemars::JsonSchema, tsify::Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
pub struct UpdateProfileRequest {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bio: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, schemars::JsonSchema, tsify::Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
pub struct UpdateProfileResponse {
    pub success: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone, schemars::JsonSchema, tsify::Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
pub struct Profile {
    pub id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bio: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub display_name_slug: Option<String>,
    pub notifications_enabled: bool,
    pub created_at: String,
    pub updated_at: String,
}
