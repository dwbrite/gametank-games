mod games;
mod auth;

use std::env;
use tokio;
use axum::{debug_handler, routing::get, Json, Router};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::{get_service};
use sqlx::{migrate, PgPool};
use std::sync::Arc;
use axum::extract::{Request, State};
use casbin::{CoreApi, MgmtApi, RbacApi};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::json;
use tower_http::services::{ServeDir, ServeFile};
use dotenvy::dotenv;
use utoipa::OpenApi;
use crate::auth::{authn_keycloak_middleware, init_casbin, init_keycloak, Casbin, Keycloak};

// #[derive(OpenApi)]
// #[openapi(paths(upload_game), components(schemas(GameEntry)))]
// pub struct ApiDoc;


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

    let keycloak = init_keycloak().await;
    let casbin = init_casbin(database_url).await;

    // reqwest http client
    let client = Arc::new(Client::new());


    // route endpoints
    let api_router = Router::new()
        .route("/user-info", get(get_user_info)) // and then is enforced
        .layer(axum::middleware::from_fn(authn_keycloak_middleware)) // user validation applies first
        .with_state((client, casbin, keycloak));

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

#[derive(Serialize, Deserialize, Debug, Clone)]
struct UserInfo {
    sub: String,
    preferred_username: String,
    email: String,
}

type MaybeUserInfo = Option<UserInfo>;

#[debug_handler]
async fn get_user_info(
    State((client, casbin, keycloak)): State<(Arc<Client>, Casbin, Keycloak)>,
    request: Request,
) -> impl IntoResponse {
    let user_info = request.extensions().get::<MaybeUserInfo>().cloned();

    // Return a unified type for all match arms
    let body = match user_info {
        Some(maybe_user) => json!(maybe_user), // Serialize UserInfo into JSON
        None => json!({ "message": "Guest user" }),
    };

    (StatusCode::OK, Json(body))
}



