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
use axum::extract::{Request, State};
use reqwest::Client;
use serde_json::json;
use tower_http::services::{ServeDir, ServeFile};
use dotenvy::dotenv;
use auth::authn_keycloak::KeycloakUserInfo;
use crate::auth::{authn_keycloak_middleware, init_casbin, init_keycloak, Casbin, KeycloakClient};
use crate::games::create_game::{create_game};
use crate::games::list_public_games;
use crate::games::patch_game::patch_game;

pub struct AppState {
    pub keycloak: KeycloakClient,
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
        keycloak: init_keycloak().await,
        casbin: init_casbin(database_url).await,
        pg_pool: pool,
        reqwest: Client::new(),
    });

    // route endpoints
    let api_router = Router::new()
        .route("/user-info", get(get_user_info))
        .route("/games", get(list_public_games))
        .route("/games", post(create_game))
        .route("/games/{game_id}", post(patch_game))
        .layer(axum::middleware::from_fn_with_state(appstate.clone(), authn_keycloak_middleware))
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



