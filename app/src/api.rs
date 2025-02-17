use reqwasm::http::{Request, Response};
use serde::{Deserialize, Serialize};
use shared::{
    model::{Paste, PasteSummary},
    PasteId, UserPasteId,
};

use crate::{Error, Result};

#[derive(Debug, Deserialize)]
pub struct ErrorResponse {
    pub code: u16,
    pub message: String,
}

#[derive(Serialize)]
pub struct CreatePaste<'a> {
    pub as_user: bool,
    pub content: &'a str,
    pub title: &'a str,
    // let the server side take care of validation here
    // the ui already tries validating and disabling the button,
    // but it's overall easier to ignore potential errors here and
    // just let the backend raise them
    #[serde(skip_serializing_if = "str::is_empty")]
    pub custom_id: &'a str,
    pub id: Option<&'a PasteId>,
    pub pinned: bool,
    pub private: bool,
}

#[allow(dead_code)] // Only used in !SSR
pub async fn create_paste(content: CreatePaste<'_>) -> Result<PasteId> {
    let _in_flight = crate::progress::start_request();
    let resp = Request::post("/api/internal/paste/")
        .body(serde_json::to_string(&content)?)
        .send()
        .await?;

    if !resp.ok() {
        return Err(handle_error_response(resp).await);
    }

    Ok(resp.json::<PasteId>().await?)
}

pub async fn get_paste(id: &PasteId) -> Result<Paste> {
    let _in_flight = crate::progress::start_request();
    let path = id.to_json_url();

    let resp = Request::get(&path).send().await?;

    if resp.status() == 404 {
        return Err(Error::NotFound("paste", id.to_string()));
    }

    if !resp.ok() {
        return Err(handle_error_response(resp).await);
    }

    Ok(resp.json().await?)
}

pub async fn delete_paste(id: &UserPasteId) -> Result<()> {
    let _in_flight = crate::progress::start_request();
    let resp = Request::delete(&format!("/api/internal/paste/{id}"))
        .send()
        .await?;

    if !resp.ok() {
        return Err(handle_error_response(resp).await);
    }

    Ok(())
}

pub async fn get_user(user: &str) -> Result<Vec<PasteSummary>> {
    let _in_flight = crate::progress::start_request();
    let resp = Request::get(&format!("/api/internal/user/{user}"))
        .send()
        .await?;

    if resp.status() == 404 {
        return Err(Error::NotFound("user", user.to_string()));
    }

    if !resp.ok() {
        return Err(handle_error_response(resp).await);
    }

    Ok(resp.json().await?)
}

async fn handle_error_response(resp: Response) -> Error {
    if let Ok(err) = resp.json::<ErrorResponse>().await {
        Error::ApiError(err.code, err.message)
    } else {
        Error::UnhandledStatus(resp.status(), resp.status_text())
    }
}
