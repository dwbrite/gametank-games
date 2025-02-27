#![deny(unused_must_use)]
#![warn(clippy::unwrap_used)]
#![warn(clippy::expect_used)]

mod games;
mod auth;
mod darn;

use std::env;
use axum::{debug_handler, routing::get, Extension, Json, Router};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::{get_service, post};
use sqlx::{migrate, PgPool};
use std::sync::Arc;
use axum::extract::{Path, Request, State};
use axum_core::extract::DefaultBodyLimit;
use reqwest::Client;
use serde_json::json;
use tower_http::services::{ServeDir, ServeFile};
use dotenvy::dotenv;
use http::Method;
use keycloak::types::TypeString;
use tower_http::limit::{RequestBodyLimit, RequestBodyLimitLayer};
use uuid::Uuid;
use auth::authn_keycloak::KeycloakUserInfo;
use darn_authorize_macro::authorize;
use crate::auth::{authn_keycloak_middleware, init_casbin, Casbin, KeycloakClient};
use crate::games::create_game::{create_game};
use crate::games::get_game::get_game;
use crate::games::{list_public_games, GameEntryData};
use crate::games::patch_game::patch_game;

pub struct AppState {
    pub casbin: Casbin,
    pub pg_pool: PgPool,
    pub reqwest: Client,
}

#[tokio::main]
#[allow(clippy::expect_used, clippy::unwrap_used)]
async fn main() {
    // TODO: Gracefully handle database/keycloak connection failures by disabling parts of the monolith.
    // Initialize service
    tracing_subscriber::fmt::init();
    dotenv().ok();
    //
    // let openapi = ApiDoc::openapi();

    // expect/unwrap justified for initialization
    let database_url: String = env::var("DATABASE_URL").unwrap();
    let pool = PgPool::connect(&database_url).await.unwrap();
    migrate!().run(&pool).await.expect("Failed to run migrations");

    let appstate = Arc::new(AppState {
        casbin: init_casbin(database_url).await,
        pg_pool: pool,
        reqwest: Client::new(),
    });

    use tower_http::cors::{CorsLayer, Any};
    
    // route endpoints
    let api_router = Router::new()
        .route("/user-info", get(get_user_info))
        .route("/games/:game_id", post(patch_game))
        .route("/games/:game_id", get(get_game))
        .route("/games", get(list_public_games))
        .route("/games", post(create_game))
        .layer(axum::middleware::from_fn_with_state(appstate.clone(), authn_keycloak_middleware))
        .layer(DefaultBodyLimit::max(10 * 1024 * 1024)) // TODO: this may be 1000x too big
        .layer(RequestBodyLimitLayer::new(10 * 1024 * 1024))
        .layer(CorsLayer::new()
            .allow_origin(Any) // Adjust your frontend port
            .allow_methods([Method::GET, Method::POST])
            .allow_headers(Any))
        .with_state(appstate.clone());

    // build our application with a route
    let app = Router::new()
        .nest("/api", api_router)
        .nest_service("/static",
            get_service(ServeDir::new("./target/ui"))
        )
        .fallback(
            get_service(ServeFile::new("./target/ui/index.html"))
        );


    // expect/unwrap justified for initialization
    let listener = tokio::net::TcpListener::bind("0.0.0.0:41123").await.expect("Can not bind to address");
    axum::serve(listener, app).await.unwrap();
}

#[debug_handler]
async fn get_user_info(
    State(_app): State<Arc<AppState>>,
    Extension(user_info): Extension<KeycloakUserInfo>,
    _request: Request,
) -> impl IntoResponse {
    // TODO: get user roles
    let body = json!(user_info);
    (StatusCode::OK, Json(body))
}


// #[debug_handler]
// async fn get_user_info_by_id(
//     State(app_state): State<Arc<AppState>>,
//     Extension(user_info): Extension<KeycloakUserInfo>,
//     Path(game_id): Path<Uuid>,
//     _request: Request,
// ) -> impl IntoResponse {
//
//     let result = app_state.keycloak.admin.realm_users_with_user_id_get(
//         &app_state.keycloak.realm,
//         &self.author.to_string(),
//         None
//     ).await.unwrap_or_default().username.unwrap_or(TypeString::from("unknown")).to_string();
// }
