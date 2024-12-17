use crate::handlers::favorite::FavoriteApi;
use crate::handlers::folder::FolderApi;
use crate::handlers::image::ImageApi;
use crate::handlers::tag::TagApi;
use crate::handlers::user::UserApi;
use crate::state::AppState;
use axum::{routing::get, Json, Router};
use utoipa::{
    openapi::security::{ApiKey, ApiKeyValue, SecurityScheme},
    Modify, OpenApi,
};
use utoipa_scalar::{Scalar, Servable as ScalarServable};

pub mod favorite;
pub mod folder;
pub mod health_check;
pub mod image;
pub mod tag;
pub mod user;

pub fn create_router() -> Router<AppState> {
    #[derive(OpenApi)]
    #[openapi(
        nest(
            (path = "/favorites", api = FavoriteApi),
            (path = "/folders", api = FolderApi),
            (path = "/images", api = ImageApi),
            (path = "/users", api = UserApi),
            (path = "/tags", api = TagApi)
        ),
        tags(
            (name = "flan", description = "Flan OpenAPI documentation")
        )
    )]
    struct ApiDoc;

    #[allow(dead_code)]
    struct SecurityAddon;

    impl Modify for SecurityAddon {
        fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
            if let Some(components) = openapi.components.as_mut() {
                components.add_security_scheme(
                    "access_key",
                    SecurityScheme::ApiKey(ApiKey::Header(ApiKeyValue::new("access_key"))),
                );
                components.add_security_scheme(
                    "admin_key",
                    SecurityScheme::ApiKey(ApiKey::Header(ApiKeyValue::new("admin_key"))),
                )
            }
        }
    }

    let api_router = Router::new()
        .nest("/users", user::router())
        .nest("/folders", folder::router())
        .nest("/images", image::router())
        .nest("/tags", tag::router())
        .nest("/favorites", favorite::router());

    let images_router = Router::new().route("/:file_id", get(image::get::get_image_handler));

    let oapi_router = Router::new()
        .merge(Scalar::with_url("/scalar", ApiDoc::openapi()))
        .route("/openapi.json", get(|| async { Json(ApiDoc::openapi()) }));

    Router::new()
        .route("/health", get(health_check::health_handler))
        .nest("/api-docs", oapi_router)
        .nest("/api", api_router)
        .nest("/images", images_router)
}
