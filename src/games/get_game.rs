use std::sync::Arc;
use axum::{Extension, Json};
use axum::body::Bytes;
use axum::extract::{Path, State};
use axum_core::response::IntoResponse;
use chrono::{DateTime, Utc};
use http::{header, HeaderMap, HeaderValue, StatusCode};
use serde::{Deserialize, Serialize};
use sqlx::query_scalar;
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


#[utoipa::path(
    get,
    path = "/games/{game_id}/rom",
    responses(
        (status = 200, description = "Game ROM retrieved successfully", content_type = "application/octet-stream"),
        (status = 403, description = "Access denied"),
        (status = 500, description = "Internal server error")
    )
)]
#[authorize(GamePermissions::View, |game_id: &Uuid| { Darn::with_namespace(GAME_NS, &game_id.to_string()) })]
pub async fn get_game_rom(
    State(app_state): State<Arc<AppState>>,
    Extension(user_info): Extension<KeycloakUserInfo>,
    Path(game_id): Path<Uuid>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    // Fetch game ROM as binary
    let game_rom = query_scalar!(
        r#"
        SELECT game_rom
        FROM game_entries
        WHERE game_id = $1
        "#,
        game_id
    )
        .fetch_one(&app_state.pg_pool)
        .await;

    match game_rom {
        Ok(rom_data) => {
            // Prepare headers
            let mut headers = HeaderMap::new();
            headers.insert(header::CONTENT_TYPE, HeaderValue::from_static("application/octet-stream"));
            headers.insert(
                header::CONTENT_DISPOSITION,
                // TODO: rename filename to not just be the game_id
                HeaderValue::from_str(&format!("attachment; filename=\"{}.gtr\"", game_id))
                    .unwrap_or_else(|_| HeaderValue::from_static("attachment; filename=\"game_rom.gtr\"")),
            );

            // Return file response
            Ok((StatusCode::OK, headers, Bytes::from(rom_data)))
        }
        Err(e) => {
            eprintln!("Error fetching game ROM: {:?}", e);
            Err((StatusCode::INTERNAL_SERVER_ERROR, "Failed to retrieve game ROM.".to_string()))
        }
    }
}
