#![allow(unused)]

mod games;
mod auth;
mod darn;

use std::env;
use std::fmt::{Display, Formatter};
use tokio;
use axum::{debug_handler, routing::get, Extension, Json, Router};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::{get_service, post};
use sqlx::{migrate, PgPool};
use std::sync::Arc;
use axum::extract::{Request, State};
use casbin::{CoreApi, MgmtApi, RbacApi};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::json;
use tower_http::services::{ServeDir, ServeFile};
use dotenvy::dotenv;
use strum_macros::Display;
use utoipa::{OpenApi, ToSchema};
use crate::auth::{authn_keycloak_middleware, init_casbin, init_keycloak, Casbin, IntoDarnWithContext, KeycloakClient};
use crate::darn::{DarNS, Darn};
use crate::games::create_game;
// #[derive(OpenApi)]
// #[openapi(paths(upload_game), components(schemas(GameEntry)))]
// pub struct ApiDoc;

pub struct AppState {
    pub keycloak: KeycloakClient,
    pub casbin: Casbin,
    pub pg_pool: PgPool,
    pub reqwest: Client,
}

#[tokio::main]
async fn main() {
    // Initialize service
    tracing_subscriber::fmt::init();
    dotenv().ok();
    //
    // let openapi = ApiDoc::openapi();

    let database_url: String = env::var("DATABASE_URL").unwrap().into();
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
        .route("/games", post(create_game))
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

    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind("0.0.0.0:41123").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

#[derive(ToSchema, Serialize, Deserialize, Debug, Clone)]
pub struct UserInfo {
    pub sub: String, // TODO: don't rename :))
    pub preferred_username: String,
    pub email: String,
}

pub const USER_NS: DarNS = DarNS("user");

impl From<&UserInfo> for Darn {
    fn from(user: &UserInfo) -> Self {
        USER_NS.new_child(&user.sub)
    }
}

pub type MaybeUserInfo = Option<UserInfo>;

#[debug_handler]
async fn get_user_info(
    State(app): State<Arc<AppState>>,
    Extension(user_info): Extension<UserInfo>,
    request: Request,
) -> impl IntoResponse {

    // Return a unified type for all match arms
    let body = json!(user_info);

    (StatusCode::OK, Json(body))
}



