use std::sync::Arc;
use axum::{Extension, Json};
use axum::extract::{Path, State};
use chrono::{DateTime, Utc};
use http::StatusCode;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use darn_authorize_macro::authorize;
use crate::AppState;
use crate::auth::{KeycloakClient, KeycloakUserInfo};
use crate::darn::Darn;
use crate::games::{GameEntryData, GameEntryMetadata, GameEntryPatch, GAME_NS};
use crate::games::game_roles::GamePermissions;

#[derive(Serialize)]
pub struct HumanGameEntryData {
    pub game_id: Uuid,
    pub game_name: String,
    pub author_id: Uuid,
    pub author: String,
    pub description: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub game_rom: Vec<u8>,
    pub public_access: bool,
}

#[utoipa::path(
    get,
    path = "/games/{game_id}",
    responses(
        (status = 200, description = "Game got successfully", body = GameEntryData),
        (status = 403, description = "Access denied"),
        (status = 500, description = "Internal server error")
    )
)]
#[authorize(GamePermissions::View, |game_id: &Uuid| { Darn::with_namespace(GAME_NS, &game_id.to_string()) })]
pub async fn get_game(
    State(app_state): State<Arc<AppState>>,
    Extension(user_info): Extension<KeycloakUserInfo>,
    Path(game_id): Path<Uuid>,
) -> Result<(StatusCode, Json<HumanGameEntryData>), (StatusCode, String)> {
    let game_result = sqlx::query_as!(
        GameEntryData,
        r#"
        SELECT *
        FROM game_entries
        WHERE game_id = $1
        "#,
        game_id
    )
        .fetch_one(&app_state.pg_pool)
        .await;

    match game_result {
        Ok(game_entry) => {
            let output = HumanGameEntryData {
                game_id,
                game_name: game_entry.game_name,
                author_id: game_entry.author,
                author: KeycloakClient::get_username(game_entry.author.to_string()).await,
                description: game_entry.description,
                created_at: game_entry.created_at,
                updated_at: game_entry.updated_at,
                game_rom: game_entry.game_rom,
                public_access: game_entry.public_access,
            };

            Ok((StatusCode::OK, Json(output)))
        }
        Err(e) => {
            eprintln!("Error fetching games: {:?}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to retrieve games.".to_string(),
            ))
        }
    }
}