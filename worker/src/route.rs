use crate::assets;
use shared::model::PasteId;
use worker::{Method, Request};

#[derive(Debug, Clone)]
pub enum Route {
    App(app::Route),
    Api(Api),
    Asset,
    NotFound,
}

impl Route {
    pub fn new(req: &Request) -> Self {
        use sycamore_router::Route;

        let path = req.path();

        if req.method() == Method::Get {
            if assets::is_asset_path(&path) {
                return Self::Asset;
            }

            let app = app::Route::match_path(&path);
            if !matches!(app, app::Route::NotFound) {
                return Self::App(app);
            }
        }

        match req.method() {
            Method::Get => {
                let route = GetEndpoints::match_path(&path);
                if !matches!(route, GetEndpoints::NotFound) {
                    return Self::Api(Api::Get(route));
                }
            }
            Method::Post => {
                let route = PostEndpoints::match_path(&path);
                if !matches!(route, PostEndpoints::NotFound) {
                    return Self::Api(Api::Post(route));
                }
            }
            Method::Delete => {
                let route = DeleteEndpoints::match_path(&path);
                if !matches!(route, DeleteEndpoints::NotFound) {
                    return Self::Api(Api::Delete(route));
                }
            }
            _ => (),
        }

        Self::NotFound
    }
}

#[derive(Debug, Clone)]
pub enum Api {
    Get(GetEndpoints),
    Post(PostEndpoints),
    Delete(DeleteEndpoints),
}

#[derive(sycamore_router::Route, strum::IntoStaticStr, Debug, Clone)]
pub enum GetEndpoints {
    #[to("/oembed.json")]
    Oembed,
    #[to("/api/internal/user/<user>")]
    User(String),
    #[to("/<id>/raw")]
    Paste(String),
    #[to("/u/<name>/<id>/raw")]
    UserPaste(String, String),
    #[to("/<id>/json")]
    PasteJson(String),
    #[to("/u/<name>/<id>/json")]
    UserPasteJson(String, String),
    /// Path of Building endpoint for importing builds.
    /// This supports the anonymous and user scoped paste IDs.
    /// User scoped paste IDs are used in `pob://` protocol links.
    /// Anonymous paste IDs are coming from importing an anonymous build URL in PoB.
    #[to("/pob/<id>")]
    PobPaste(PasteId),
    /// Path of Building endpoint for importing user paste URLs.
    #[to("/pob/u/<name>/<id>")]
    PobUserPaste(String, String),
    #[to("/login")]
    Login(),
    #[to("/oauth2/authorization/poe")]
    Oauht2Poe(),
    #[not_found]
    NotFound,
}

#[derive(sycamore_router::Route, strum::IntoStaticStr, Debug, Clone)]
pub enum PostEndpoints {
    #[to("/api/internal/paste/")]
    Upload(),
    #[to("/pob/")]
    PobUpload(),
    #[not_found]
    NotFound,
}

#[derive(sycamore_router::Route, strum::IntoStaticStr, Debug, Clone)]
pub enum DeleteEndpoints {
    #[to("/api/internal/paste/<id>")]
    DeletePaste(PasteId),
    #[not_found]
    NotFound,
}
