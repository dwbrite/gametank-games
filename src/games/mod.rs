mod game_roles;
pub mod create_game;
pub mod patch_game;

use std::sync::Arc;
use axum::{Extension, Json};
use axum::extract::State;
use sqlx::FromRow;
use utoipa::ToSchema;

use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use http::StatusCode;
use darn_authorize_macro::authorize;
use crate::AppState;
use crate::auth::{DefaultNamespace, KeycloakUserInfo, SitePermissions, SiteRoles};

#[derive(Debug, Deserialize, ToSchema)]

pub struct GameEntryCreate {
    pub game_name: String,
    pub description: String,
    pub game_rom: Vec<u8>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct GameEntryPatch {
    pub _game_name: Option<String>,
    pub _description: Option<String>,
    pub _game_rom: Option<Vec<u8>>,
}

#[derive(Debug, FromRow, ToSchema, Serialize, Deserialize)]
pub struct GameEntryData {
    pub game_id: Uuid,
    pub game_name: String,
    pub author: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub game_rom: Vec<u8>,
}


#[derive(Debug, FromRow, ToSchema, Serialize, Deserialize)]
pub struct GameEntryMetadata {
    pub game_id: Uuid,
    pub game_name: String,
    pub author: Uuid,
    pub description: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

const GAME_NS: &str = "game";

#[utoipa::path(
    get,
    path = "/games",
    responses(
        (status = 200, description = "Here's a list of games!!!", body = [GameEntryMetadata]),
        (status = 403, description = "Forbidden"),
        (status = 500, description = "Internal Server Error")
    )
)]
#[authorize(SitePermissions::ViewPublic, || { SiteRoles::default_namespace() })]
pub async fn list_public_games(
    State(app_state): State<Arc<AppState>>,
    Extension(user_info): Extension<KeycloakUserInfo>,
) -> Result<(StatusCode, Json<Vec<GameEntryMetadata>>), (StatusCode, String)> {
    let games_result = sqlx::query_as!(
        GameEntryMetadata,
        r#"
        SELECT
            game_id,
            game_name,
            description,
            author,
            created_at,
            updated_at
        FROM
            game_entries
        ORDER BY
            created_at DESC
        "#
    )
        .fetch_all(&app_state.pg_pool)
        .await;

    match games_result {
        Ok(games) => Ok((StatusCode::OK, Json(games))),
        Err(e) => {
            // Log the error for debugging purposes
            eprintln!("Error fetching games: {:?}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to retrieve games.".to_string(),
            ))
        }
    }
}

