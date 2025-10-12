use crate::utils::hit_ai_server;
use language_utils::profile::{
    FollowRequest, FollowResponse, FollowStatus, Profile, UpdateProfileRequest,
    UpdateProfileResponse, UserLanguageStats,
};
use wasm_bindgen::prelude::*;

#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
pub async fn get_profile_by_id(user_id: String) -> Result<JsValue, JsValue> {
    let response = hit_ai_server(
        fetch_happen::Method::GET,
        &format!("/profile?id={user_id}"),
        None::<()>,
        None,
    )
    .await
    .map_err(|e| JsValue::from_str(&format!("Request error: {e:?}")))?;

    if !response.ok() {
        return Err(JsValue::from_str(&format!(
            "HTTP error: {}",
            response.status()
        )));
    }

    let profile: Profile = response
        .json()
        .await
        .map_err(|e| JsValue::from_str(&format!("Response parsing error: {e:?}")))?;

    serde_wasm_bindgen::to_value(&profile)
        .map_err(|e| JsValue::from_str(&format!("Serialization error: {e:?}")))
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
pub async fn get_profile_by_slug(slug: String) -> Result<JsValue, JsValue> {
    let response = hit_ai_server(
        fetch_happen::Method::GET,
        &format!("/profile?slug={slug}"),
        None::<()>,
        None,
    )
    .await
    .map_err(|e| JsValue::from_str(&format!("Request error: {e:?}")))?;

    if !response.ok() {
        return Err(JsValue::from_str(&format!(
            "HTTP error: {}",
            response.status()
        )));
    }

    let profile: Profile = response
        .json()
        .await
        .map_err(|e| JsValue::from_str(&format!("Response parsing error: {e:?}")))?;

    serde_wasm_bindgen::to_value(&profile)
        .map_err(|e| JsValue::from_str(&format!("Serialization error: {e:?}")))
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
pub async fn update_profile(
    display_name: Option<String>,
    bio: Option<String>,
    access_token: String,
) -> Result<UpdateProfileResponse, JsValue> {
    let request = UpdateProfileRequest { display_name, bio };

    let response = hit_ai_server(
        fetch_happen::Method::PATCH,
        "/profile",
        Some(&request),
        Some(&access_token),
    )
    .await
    .map_err(|e| JsValue::from_str(&format!("Request error: {e:?}")))?;

    if !response.ok() {
        return Err(JsValue::from_str(&format!(
            "HTTP error: {}",
            response.status()
        )));
    }

    let result: UpdateProfileResponse = response
        .json()
        .await
        .map_err(|e| JsValue::from_str(&format!("Response parsing error: {e:?}")))?;

    Ok(result)
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
pub async fn get_user_language_stats_by_id(user_id: String) -> Result<JsValue, JsValue> {
    let response = hit_ai_server(
        fetch_happen::Method::GET,
        &format!("/user-language-stats?id={user_id}"),
        None::<()>,
        None,
    )
    .await
    .map_err(|e| JsValue::from_str(&format!("Request error: {e:?}")))?;

    if !response.ok() {
        return Err(JsValue::from_str(&format!(
            "HTTP error: {}",
            response.status()
        )));
    }

    let stats: Vec<UserLanguageStats> = response
        .json()
        .await
        .map_err(|e| JsValue::from_str(&format!("Response parsing error: {e:?}")))?;

    serde_wasm_bindgen::to_value(&stats)
        .map_err(|e| JsValue::from_str(&format!("Serialization error: {e:?}")))
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
pub async fn get_user_language_stats_by_slug(slug: String) -> Result<JsValue, JsValue> {
    let response = hit_ai_server(
        fetch_happen::Method::GET,
        &format!("/user-language-stats?slug={slug}"),
        None::<()>,
        None,
    )
    .await
    .map_err(|e| JsValue::from_str(&format!("Request error: {e:?}")))?;

    if !response.ok() {
        return Err(JsValue::from_str(&format!(
            "HTTP error: {}",
            response.status()
        )));
    }

    let stats: Vec<UserLanguageStats> = response
        .json()
        .await
        .map_err(|e| JsValue::from_str(&format!("Response parsing error: {e:?}")))?;

    serde_wasm_bindgen::to_value(&stats)
        .map_err(|e| JsValue::from_str(&format!("Serialization error: {e:?}")))
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
pub async fn follow_user(user_id: String, access_token: String) -> Result<JsValue, JsValue> {
    let request = FollowRequest { user_id };

    let response = hit_ai_server(
        fetch_happen::Method::POST,
        "/follow",
        Some(&request),
        Some(&access_token),
    )
    .await
    .map_err(|e| JsValue::from_str(&format!("Request error: {e:?}")))?;

    if !response.ok() {
        return Err(JsValue::from_str(&format!(
            "HTTP error: {}",
            response.status()
        )));
    }

    let result: FollowResponse = response
        .json()
        .await
        .map_err(|e| JsValue::from_str(&format!("Response parsing error: {e:?}")))?;

    serde_wasm_bindgen::to_value(&result)
        .map_err(|e| JsValue::from_str(&format!("Serialization error: {e:?}")))
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
pub async fn unfollow_user(user_id: String, access_token: String) -> Result<JsValue, JsValue> {
    let request = FollowRequest { user_id };

    let response = hit_ai_server(
        fetch_happen::Method::POST,
        "/unfollow",
        Some(&request),
        Some(&access_token),
    )
    .await
    .map_err(|e| JsValue::from_str(&format!("Request error: {e:?}")))?;

    if !response.ok() {
        return Err(JsValue::from_str(&format!(
            "HTTP error: {}",
            response.status()
        )));
    }

    let result: FollowResponse = response
        .json()
        .await
        .map_err(|e| JsValue::from_str(&format!("Response parsing error: {e:?}")))?;

    serde_wasm_bindgen::to_value(&result)
        .map_err(|e| JsValue::from_str(&format!("Serialization error: {e:?}")))
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
pub async fn get_follow_status(
    user_id: Option<String>,
    slug: Option<String>,
    access_token: String,
) -> Result<JsValue, JsValue> {
    let query = if let Some(id) = user_id {
        format!("id={id}")
    } else if let Some(s) = slug {
        format!("slug={s}")
    } else {
        return Err(JsValue::from_str("Either user_id or slug must be provided"));
    };

    let response = hit_ai_server(
        fetch_happen::Method::GET,
        &format!("/follow-status?{query}"),
        None::<()>,
        Some(&access_token),
    )
    .await
    .map_err(|e| JsValue::from_str(&format!("Request error: {e:?}")))?;

    if !response.ok() {
        return Err(JsValue::from_str(&format!(
            "HTTP error: {}",
            response.status()
        )));
    }

    let status: FollowStatus = response
        .json()
        .await
        .map_err(|e| JsValue::from_str(&format!("Response parsing error: {e:?}")))?;

    serde_wasm_bindgen::to_value(&status)
        .map_err(|e| JsValue::from_str(&format!("Serialization error: {e:?}")))
}
